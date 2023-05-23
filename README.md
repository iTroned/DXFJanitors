# DXFJanitors
## Info
This is a software tool made for [Laiout](https://www.laiout.co/) as part of our bachelor thesis from OsloMet (Norway)

## Usage
1. Download the [dxfjanitors.exe] file
2. Make sure you have the correct python libs installed (PyO3, ezdxf & matplotlib)
3. Run

## Keybinds

1. Open: CTRL + O
2. Save: CTRL + S
3. Undo: CTRL + Z
4. Redo: CTRL + SHIFT + Z
5. Zoom: CTRL + + & CTRL + - / ALT + Scroll


## Clone the repository:

This is a guide on how to access and use the code after cloning the repository from link.
1.	Download VSCode: https://code.visualstudio.com/
2.	Install build-tools for Visual Studio with the Visual C++ option: https://visualstudio.microsoft.com/downloads/?q=build+tools (located at the bottom of the site). When downloading, choose the Visual C++ option.
3.	Install rustup. It is an installer for Rust: https://rustup.rs/
4.	Install the rust-analyzer extension in VSCode: https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer
5.	Install Python: https://www.python.org/downloads/windows/
6.	Clone the Repository from: https://github.com/iTroned/DXFJanitors
7.	Open the project in VSCode.
8.	Wait for the rust-analyzer to load.
9.	If error message says: “error: no Python 3.x interpreter found.”
  a.	Find your local Python interpreter’s file location.
  b.	Find the Python310.dll file:
  c.	Copy the Python310.dll file into the project folder Main/target/debug/:
  d.	Re-open the project or restart Rust-analyzer. 
      If the error message still shows, try these steps:
      1.	Uninstall the Python Interpreter.
      2.	Download Python version 3.10.10.
      3.	Add the Python310.dll file from the new downloaded files to Main/target/debug/
      4.	Specify the Python Interpreter in VSCode with “Ctrl + Shift + P” and search for “Select Python Interpreter”.
      5.	Browse the internet for solutions

      Alternatively create a new Python environment. 


10.	These libraries need to be installed on the Python interpreter or via an environment:
Matplotlib: https://matplotlib.org/stable/users/installing/index.html
Ezdxf: https://pypi.org/project/ezdxf/


11.	Use the following commands in terminal to explore the project:
All commands can be found here: https://doc.rust-lang.org/cargo/commands/build-commands.html

1.	“cd main”: All files are located in the "main" folder.
 
2.	Start the program with “cargo run”.

3.	Run all unit and integration tests with “cargo test”.
 

 
 


