mod klasse;

mod id;

mod save_file;

pub mod schueler_file;
pub mod schueler_liste_file;

pub use klasse::Klasse;

pub use id::projekt_id::ProjektId;
pub use id::schueler_id::SchuelerId;

pub use save_file::SaveFile;

pub use save_file::SaveFileKlasse;
pub use save_file::SaveFileProjekt;
pub use save_file::SaveFileSchueler;
pub use save_file::SaveFileStufe;
pub use save_file::SaveFileZuordnung;
