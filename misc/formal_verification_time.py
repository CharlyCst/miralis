import re
import subprocess
import time

def extract_kani_names(file_path):
    pattern = re.compile(
        r"#\[cfg_attr\(kani, kani::proof\)\]\s*"
        r"#\[cfg_attr\(test, test\)\]\s*"
        r"pub fn (\w+)\s*\("
    )

    with open(file_path, 'r') as file:
        content = file.read()

    return pattern.findall(content)

def time_cargo_run(subcommand):
    command = ["cargo", "run", "--", "verify", subcommand]

    start_time = time.time()

    try:
        with open('/dev/null', 'w') as devnull:
            subprocess.run(command, check=True, stdout=devnull, stderr=devnull)
    except subprocess.CalledProcessError as e:
        print(f"Command failed with return code {e.returncode}")

    duration = time.time() - start_time

    print(f"{subcommand} verification completed in {duration:.2f} seconds.")

    return duration

if __name__ == "__main__":
    print("Please run the script from the miralis folder, otherwise it won't work.")

    file_path = "model_checking/src/lib.rs"
    functions = extract_kani_names(file_path)

    for func in functions:
        time_cargo_run(func)
