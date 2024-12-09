import os
import subprocess

def run_bash_command(command):
    """
    Runs a Bash command and prints its output.

    Args:
        command (str): The Bash command to execute.
    """
    try:
        result = subprocess.run(command, shell=True, check=True, text=True, capture_output=True)
        print(result.stdout)
    except subprocess.CalledProcessError as e:
        print(f"Error: Command '{command}' failed with error:\n{e.stderr}")

values = []

def read_file_line_by_line(file_path):
    """
    Reads a file line by line and parses the documentation.

    Args:
        file_path (str): Path to the file to be read.
    """
    with open(file_path, 'r') as file:
        header = file.readline()

        if header.startswith("== ") and "EID " in header:
            fid = header.split("#")[1].split(" ")[0].split(")")[0]

        for line in file:
            if line.startswith("=== ") and "FID " in line:
                eid = line.split("#")[1].split(" ")[0].split(")")[0]
                # Consume the next two lines
                file.readline()
                file.readline()
                file.readline()
                curr = ""
                line = ""
                while True:
                    line = file.readline()

                    if line.strip() == "----" or line.strip() == "```":
                        break

                    curr += line

                malus = 0
                if "void" in curr:
                    malus += 1

                values.append((fid, eid, curr.count(",") + 1 - malus))


run_bash_command("git clone https://github.com/riscv-non-isa/riscv-sbi-doc/")

for root, _, files in os.walk("riscv-sbi-doc/src"):
    for file in files:
        file_path = os.path.join(root, file)
        read_file_line_by_line(file_path)

code = "// ———————————————————————————————— Filtering rules for ecall - automatically generated ———————————————————————————————— //\n\n"
code += "fn get_nb_input_args(eid: usize, fid: usize) -> usize {\n"
code += "   match(eid,fid) {\n"

for v in values:
    code += "       ({},{}) => {},\n".format(v[0], v[1], v[2])

code += "       _ => 0,\n"
code += "   }\n"
code += "}"

print(code)

run_bash_command("rm -rf riscv-sbi-doc")