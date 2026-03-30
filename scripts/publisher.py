import socket
import sys


def send_packet(sock: socket.socket, topic: bytes, message: bytes) -> None:
    packet = (
        len(topic).to_bytes(4, "big") +
        topic +
        len(message).to_bytes(4, "big") +
        message
    )
    sock.sendall(packet)


def main() -> None:
    host = "127.0.0.1"
    port = 8888

    if len(sys.argv) >= 2:
        port = int(sys.argv[1])

    sock = socket.create_connection((host, port))
    print(f"connected to publisher port {port}")

    try:
        while True:
            topic_text = input("topic (empty to quit): ").strip()
            if not topic_text:
                break

            message_text = input("message: ")
            send_packet(sock, topic_text.encode("utf-8"), message_text.encode("utf-8"))
            print("sent")
    finally:
        sock.close()
        print("publisher closed")


if __name__ == "__main__":
    main()