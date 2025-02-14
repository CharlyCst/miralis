# This files generates the filtering rules from the rust sbi doc. For each fid / eid pairs, it generates a function that tells how many filter you should allow starting from a0 (in addition to a6 and a7 for the eid / fid fields)
# The script runs out of the box. It will clone the github repository, generate the function and remove the repository

import os
import subprocess

def run_bash_command(command):
    """
    Runs a Bash command and prints its output.

    Args:
        command (str): The Bash command to execute.
    """
    try:
        subprocess.run(command, shell=True, check=True, text=True, capture_output=True)
    except subprocess.CalledProcessError as e:
        print(f"Error: Command '{command}' failed with error:\n{e.stderr}")

def read_file_line_by_line(file_path):
    """
    Reads a file line by line and parses the documentation.

    Args:
        file_path (str): Path to the file to be read.
    """
    output = []
    with open(file_path, 'r') as file:
        header = file.readline()

        if header.startswith("== ") and "EID " in header:
            eid = header.split("#")[1].split(" ")[0].split(")")[0]

        is_legacy = False
        if header.strip() == "== Legacy Extensions (EIDs #0x00 - #0x0F)":
            is_legacy = True

        for line in file:
            if line.startswith("=== ") and ("FID " in line or "EID " in line) :
                fid = line.split("#")[1].split(" ")[0].split(")")[0]
                # Consume the next two lines
                file.readline()
                file.readline()
                curr = file.readline()
                while True:
                    line = file.readline()

                    if line.strip() == "----" or line.strip() == "```":
                        break

                    curr += line

                malus = 0
                if "void" in curr:
                    malus += 1

                if is_legacy:
                    output.append((fid, "0", curr.count(",") + 1 - malus))
                else:
                    output.append((eid, fid, curr.count(",") + 1 - malus))


    return output

def generate_function(values):
    """
    Generates the rust function

    Args:
        values: a list of tuples with (fid, eid, number_of_registers to allow)
    """
    code = "// ———————————————————————————————— Filtering rules for ecall - automatically generated ———————————————————————————————— //\n\n"
    code += "fn get_nb_input_args(eid: usize, fid: usize) -> usize {\n"
    code += "   match(eid,fid) {\n"

    for v in values:
        code += "       ({},{}) => {},\n".format(v[0], v[1], v[2])

    code += "       _ => 0,\n"
    code += "   }\n"
    code += "}"

    return code

if __name__ == "__main__":
    run_bash_command("git clone https://github.com/riscv-non-isa/riscv-sbi-doc/")

    values = []
    for root, _, files in os.walk("riscv-sbi-doc/src"):
        for file in files:
            file_path = os.path.join(root, file)
            values = values + read_file_line_by_line(file_path)

    print(generate_function(sorted(values)))

    run_bash_command("rm -rf riscv-sbi-doc")