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


## Installation guide (until a safe version on an embedded interpreter is developed):

This is a guide on how to access and use the code after cloning the repository from link.
1.	Download VSCode: https://code.visualstudio.com/
2.	Install build-tools for Visual Studio with the Visual C++ option: https://visualstudio.microsoft.com/downloads/?q=build+tools (located at the bottom of the site). When downloading, choose the Visual C++ option.
3.	Install rustup. It is an installer for Rust: https://rustup.rs/
4.	Install the rust-analyzer extension in VSCode: https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer

5. Until a stable version of an embedded python interpreter, the user unfortunately needs to setup a local python environment for the program to use:<br>
<br> 1. Install Miniconda: https://docs.conda.io/en/latest/miniconda.html <br>
<br> 2. Open miniconda. <br>
<br> 3. Create a new environment in miniconda with: conda create --name myenv <br>
<br> 4. Activate the environment in miniconda: conda activate myenv <br>
<br> 5. Install ezdxf=1.0.2. "conda install -c conda-forge ezdxf=1.0.2" - (https://anaconda.org/conda-forge/ezdxf) <br>
<br> 6. Install matplotlib. "conda install -c conda-forge matplotlib" - (https://anaconda.org/conda-forge/matplotlib) <br>
<br> 7. Locate the folder for the created environment. Standard location is: C:\Users\"username"\miniconda3\envs <br>
<br> 8. Clone the Repository from https://github.com/iTroned/DXFJanitors <br>
<br> 9. Open the project in VSCode <br> 
<br> 10. You should get the error message: Error: No python interpreter 3.x found. <br>
Press Ctrl + Shift + P, and search Python. Pick "Python: Select Interpreter.". Locate the python.exe file from the folder C:\Users\"username"\miniconda3\envs\myenv\ and select it. <br>
<br> After selection, copy the Python310.dll file from C:\Users\"username"\miniconda3\envs\myenv\ and place it in the project folder Main/target/debug/

<br>Restart the application
<br>If the error message still appear, copy every single file from C:\Users\"username"\miniconda3\envs\myenv\ int o the project folder Main/target/debug/.


6.	Use the following commands in terminal to explore the project:
All commands can be found here: https://doc.rust-lang.org/cargo/commands/build-commands.html

1.	“cd main”: All files are located in the "main" folder.
 
2.	Start the program with “cargo run”.

3.	Run all unit and integration tests with “cargo test”.
 

 
 


