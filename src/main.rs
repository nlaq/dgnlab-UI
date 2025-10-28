use rfd::FileDialog;
use rfd::MessageDialog;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use std::process::Command;
use std::io::ErrorKind;



slint::include_modules!();



// -----------------------------------------------------------------------------
// DNGLAB INSTALLED? FUNCTION: If installed and type of installation
// -----------------------------------------------------------------------------


  fn dnglab_installed() -> bool {

	let result = if cfg!(target_os = "macos") {
        Command::new("/usr/local/bin/dnglab")
            .arg("--version")
            .output()
    } else if cfg!(target_os = "linux") {
        Command::new("dnglab")
            .arg("--version")
            .output()
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Unsupported OS"))
    };    


    match result {
      // Command executed and finished successfully (or failed to execute, but we caught the error)
      Ok(t) => {

          println!("Ok, System  installed command dnglab: {:#?}", t);
          true
      }
      Err(e) => {

          //let result =
          MessageDialog::new()
          .set_level(rfd::MessageLevel::Info) // Set dialog level (Info, Warning, Error)
          .set_title("DngLab Warning")        // Set dialog title
          .set_description("You must first install dnglab. More info: ") // Set message text
          .set_buttons(rfd::MessageButtons::Ok) // Set buttons (Ok, OkCancel, YesNo, YesNoCancel)
          .show(); // Show the dialog and get the result
          println!("DNGLab not installed: {:#?}", e);
          false

      }
    }
  }



// -----------------------------------------------------------------------------
// PROCESS FUNCTION: execute dnglab on terminal
// -----------------------------------------------------------------------------


// 2. Use the correct executable and the .args() method

fn process(args: Vec<String>, override_1: bool, recursive: bool, input: PathBuf, output_path: PathBuf) {


    let mut dnglab = Command::new("/usr/local/bin/dnglab");

    dnglab.arg("convert".to_string());
    dnglab.args(args);



    if override_1 {
      dnglab.arg("--override");
    }
    if recursive {
      dnglab.arg("--recursive");
    }
    dnglab.arg(&input);
    dnglab.arg(&output_path);

    println!("Executing command: {:?}", dnglab);/////////////////////////////////////////////////////////

    let dnglab_exec = dnglab.output();


    match dnglab_exec {
      // Command executed and finished successfully (or failed to execute, but we caught the error)
      Ok(output) => {
        if output.status.success() {
          println!("Command finished successfully.");
        } else {
          eprintln!("Command finished but returned an error status: {}", output.status);
          eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
      }
      // OS failed to start the process (e.g., command not found)
      Err(e) => {
        if e.kind() == ErrorKind::NotFound {
          eprintln!("ERROR: The command was not found. Is it installed and in your PATH?");
        } else {
          eprintln!("An unexpected error occurred while trying to run the command: {}", e);
        }
      }
    }

}


/// Checks if both input_paths (Vec) and output_path (String) are non-empty
/// and sets the 'convert_true' property on the UI accordingly.
fn check_and_enable_convert(
  ui: &MainWindow,
  input_paths: &Rc<RefCell<Vec<PathBuf>>>,
  output_path: &Rc<RefCell<PathBuf>>
) {
  // Check if the vector is NOT empty
  let input_is_set = !input_paths.borrow().is_empty();

  // Check if the output string is NOT empty
  let output_is_set = !output_path.borrow().display().to_string().is_empty();

  // Set the Slint property to true only if both conditions are met
  ui.set_convert_true(input_is_set && output_is_set);
}


// -----------------------------------------------------------------------------
// MAIN FUNCTION
// -----------------------------------------------------------------------------

fn main() -> Result<(), slint::PlatformError> {


  dnglab_installed();

  let main_window = MainWindow::new()?;

  // --- Initial Weak Handle ---
  let main_window_weak = main_window.as_weak();

  // --- Shared Mutable State (Rc<RefCell<T>>) ---
  //let output_path = Rc::new(RefCell::new(String::new()));
  let output_path: Rc<RefCell<PathBuf>> = Rc::new(RefCell::new(PathBuf::new()));
  let input_paths: Rc<RefCell<Vec<PathBuf>>> = Rc::new(RefCell::new(Vec::new()));


  // --- Rc Clones for Handlers ---

  // Input Handler Clones
  let input_paths_c_in = Rc::clone(&input_paths);
  let output_path_c_in = Rc::clone(&output_path);

  // Output Handler Clones
  let input_paths_c_out = Rc::clone(&input_paths);
  let output_path_c_out = Rc::clone(&output_path);

  // Convert Handler Clones
  let input_paths_c_conv = Rc::clone(&input_paths);
  let output_path_c_conv = Rc::clone(&output_path);


  // -----------------------------------------------------------
  // Input Handler: Select Files

  // Clone the weak handle before moving it into the closure
  let main_window_weak_input = main_window_weak.clone();
  main_window.on_input_files(move|| {

    let Some(ui) = main_window_weak_input.upgrade() else { return }; // Uses the clone

    let paths = FileDialog::new()
    .add_filter("Raw Files", &[
      "ari",
      "cr3",
      "cr2",
      "crw",
      "erf",
      "raf",
      "3fr",
      "kdc",
      "dcs",
      "dcr",
      "iiq",
      "mos",
      "mef",
      "mrw",
      "nef",
      "nrw",
      "orf",
      "rw2",
      "pef",
      "srw",
      "arw",
      "srf",
      "sr2"
    ])
    .set_directory("~/Desktop")
    .pick_files();

    let paths = match paths {
      Some(p) => p,
      None => return,
    };

    // 1. Update shared state
    *input_paths_c_in.borrow_mut() = paths.clone();

    // 2. Update UI text
    let files_count = format!("{} file(s) selected.", paths.len());
    ui.set_input_path(files_count.into());

    // 3. Check condition and update the button state
    check_and_enable_convert(&ui, &input_paths_c_in, &output_path_c_in);
  });


  // -----------------------------------------------------------
  // Output Handler: Select Folder

  // Clone the weak handle before moving it into the closure
  let main_window_weak_output = main_window_weak.clone();
  main_window.on_output_folder(move||{

    let Some(ui) = main_window_weak_output.upgrade() else { return }; // Uses the clone

    let path = FileDialog::new()
    .set_directory("~/Desktop")
    .pick_folder();

    let path = match path {
      Some(p) => p,
                               None => return,
    };

    let path_str = path;

    // 1. Update shared state
    *output_path_c_out.borrow_mut() = path_str.clone();

    // 2. Update UI text (truncation logic)
    let char_count = path_str.display().to_string().chars().count();
    if char_count <= 20 {
      ui.set_output_path(path_str.display().to_string().into());
    } else {
      let skip = char_count.saturating_sub(20);
      let path_truncated = format!("...{}", path_str.display().to_string().chars().skip(skip).collect::<String>());
      ui.set_output_path(path_truncated.into());
    }

    // 3. Check condition and update the button state
    check_and_enable_convert(&ui, &input_paths_c_out, &output_path_c_out);
  });


  // -----------------------------------------------------------
  // Convert Handler: Execute Logic

  // Clone the weak handle before moving it into the closure
  let main_window_weak_convert = main_window_weak.clone();
  main_window.on_convert_pressed(move||{
    let Some(ui) = main_window_weak_convert.upgrade() else { return };

    // Read UI settings
    let compression: String = ui.get_compression().to_string();
    let crop: String = ui.get_crop().to_string();
    let embeded: bool = ui.get_embeded();
    let override_1: bool = ui.get_override_1();
    let recursive: bool = ui.get_recursive();

    // Final access to the shared state
    let input_array = <Vec<PathBuf> as Clone>::clone(&input_paths_c_conv.borrow()).into_iter();

    let output_path = output_path_c_conv.borrow();




      let installed = dnglab_installed();

      for input in input_array {

        let args = vec![

                        //"--dng-preview".to_string(),
                        //"false".to_string(),
                        "--embed-raw".to_string(),
                        embeded.to_string(),
                        "--compression".to_string(),
                        compression.to_string(),
                        "--crop".to_string(),
                        crop.to_string(),

        ];



        //println!("{:#?}", args);

        if installed {
          process(args, override_1, recursive, input, output_path.to_path_buf()); //// ESTE ES PARA PROCESAR VERIFICADA LA INSTALACION
        }
        else {
          println!("DNG not installed");
        }
    }
  });
  main_window.run()
}
