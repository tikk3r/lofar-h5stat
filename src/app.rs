pub enum CurrentScreen {
    Main,
    Opening,
    Exiting,
}

pub enum CurrentlyEditing {
    Solset,
    Soltab,
    Information,
}

pub struct App {
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub currently_editing: CurrentlyEditing, // the optional state containing which of the key or value pair the user is editing. It is an option, because when the user is not directly editing a key-value pair, this will be set to `None`.
    pub h5parm: lofar_h5parm_rs::H5parm,
    pub solsets: Vec<lofar_h5parm_rs::SolSet>,
    pub current_solset: usize,
    pub soltabs: Vec<lofar_h5parm_rs::SolTab>,
    pub current_soltab: usize,
    pub text_buffer: String,
    pub text_scroll: u16,
}

impl App {
    pub fn new(h5parm_in: String) -> App {
        let h5 = lofar_h5parm_rs::H5parm::open(&h5parm_in, true).expect("Failed to read H5parm.");
        let ss = h5.solsets.clone();
        let st = ss[0].soltabs.clone();

        let mut app = App {
            current_screen: CurrentScreen::Main,
            currently_editing: CurrentlyEditing::Solset,
            h5parm: h5,
            solsets: ss,
            current_solset: 0,
            soltabs: st,
            current_soltab: 0,
            text_buffer: "".to_string(),
            text_scroll: 0,
        };
        app.select();
        app
    }

    pub fn toggle_editing(&mut self, forwards: bool) {
        match &self.currently_editing {
            CurrentlyEditing::Solset => {
                if forwards {
                    self.currently_editing = CurrentlyEditing::Soltab
                } else {
                    self.currently_editing = CurrentlyEditing::Information
                }
            }
            CurrentlyEditing::Soltab => {
                if forwards {
                    self.currently_editing = CurrentlyEditing::Information
                } else {
                    self.currently_editing = CurrentlyEditing::Solset
                }
            }
            CurrentlyEditing::Information => {
                if forwards {
                    self.currently_editing = CurrentlyEditing::Solset
                } else {
                    self.currently_editing = CurrentlyEditing::Soltab
                }
            }
        };
    }

    pub fn increase_soltab(&mut self) {
        match &self.currently_editing {
            CurrentlyEditing::Solset => {
                self.current_solset += 1;
                if self.current_solset >= self.solsets.len() {
                    self.current_solset = 0;
                }
            }
            CurrentlyEditing::Soltab => {
                self.current_soltab += 1;
                if self.current_soltab >= self.soltabs.len() {
                    self.current_soltab = 0;
                }
            }
            CurrentlyEditing::Information => {
                self.text_scroll += 1;
            }
        }
    }

    pub fn decrease_soltab(&mut self) {
        match &self.currently_editing {
            CurrentlyEditing::Solset => {
                if self.current_solset == 0 {
                    self.current_solset = self.solsets.len() - 1;
                } else {
                    self.current_solset -= 1;
                }
                self.update_soltabs();
            }
            CurrentlyEditing::Soltab => {
                if self.current_soltab == 0 {
                    self.current_soltab = self.soltabs.len() - 1;
                } else {
                    self.current_soltab -= 1;
                }
            }
            CurrentlyEditing::Information => {
                if self.text_scroll > 0 {
                    self.text_scroll -= 1;
                }
            }
        }
    }

    pub fn update_soltabs(&mut self) {
        match &self.currently_editing {
            CurrentlyEditing::Solset => {
                let ss = self.h5parm.solsets.clone();
                let new_soltabs = ss[self.current_solset].soltabs.clone();
                self.soltabs = new_soltabs;
                self.current_soltab = 0;
            }
            CurrentlyEditing::Soltab => {}
            _ => {}
        }
    }

    pub fn select(&mut self) {
        match &self.currently_editing {
            CurrentlyEditing::Solset => {
                let h5 = self.h5parm.clone();
                let ss = h5.solsets;
                let mut buf = "".to_string();
                let headline = format!(
                    "{:<21} {:<16} {:<13} {:<13}",
                    "Solutions", "Type", "Polarisations", "Antennas"
                );
                buf.push_str(&headline);
                buf.push_str("\n");
                for st in ss[self.current_solset].soltabs.iter() {
                    let stationlist = st.get_antennas();
                    let cs = stationlist
                        .iter()
                        .filter(|s| s.starts_with("CS") || s.starts_with("ST"))
                        .collect::<Vec<_>>();
                    let rs = stationlist
                        .iter()
                        .filter(|s| s.starts_with("RS"))
                        .collect::<Vec<_>>();
                    let is = stationlist
                        .iter()
                        .filter(|s| !s.starts_with("CS") && !s.starts_with("RS"))
                        .collect::<Vec<_>>();
                    let line = format!(
                        "{:<21} {:<16} {:<13} {} ({:>2}/{}/{})",
                        st.name,
                        st.get_type(),
                        st.get_polarisations().to_vec().join(","),
                        st.get_antennas().len(),
                        cs.len(),
                        rs.len(),
                        is.len(),
                    );
                    buf.push_str(&line);
                    buf.push_str("\n");
                }
                self.text_buffer = buf;
            }
            CurrentlyEditing::Soltab => self.select_soltab(),
            _ => {}
        }
    }

    fn select_soltab(&mut self) {
        let h5 = self.h5parm.clone();
        let ss = &h5.solsets[self.current_solset];
        let st = &ss.soltabs[self.current_soltab];

        let axes = st.get_axes();

        let times = st.get_times();
        let dt = times[1] - times[0];

        let freqs = st.get_frequencies().unwrap_or_default();
        let df = if freqs.len() > 1 {
            freqs[1] - freqs[0]
        } else {
            0.0
        };

        let stations = st
            .get_antennas()
            .to_vec()
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        let flagged_fraction = st.get_flagged_fraction();

        let dirs = st.get_directions().unwrap_or_default();
        let dir = if dirs.len() >= 1 {
            dirs.to_vec()[0].to_string()
        } else {
            "AAAAAAAAH BUG".to_string()
        };

        let mut buf = "".to_string();
        buf.push_str(&format!("Dimensions: {}\n", axes.join(", ")));
        buf.push_str("\n");
        buf.push_str(&format!("Directions: {}\n", dir));
        buf.push_str("\n");
        buf.push_str(&format!(
            "Overall flagged fraction: {:.2}%\n",
            flagged_fraction * 100.0
        ));
        buf.push_str("\n");
        buf.push_str(&format!("Time interval: {} s\n", dt));
        if df > 0.0 {
            buf.push_str(&format!("Frequency interval: {} Hz\n", df));
        } else {
            buf.push_str(&"Frequency interval: N/A\n");
        }
        buf.push_str("\nStations present:\n");
        buf.push_str(&stations);

        self.text_buffer = buf;
    }
}
