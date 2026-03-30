import socket
import sys


def recv_exact(sock: socket.socket, n: int) -> bytes:
    data = b""
    while len(data) < n:
        chunk = sock.recv(n - len(data))
        if not chunk:
            raise ConnectionError("socket closed")
        data += chunk
    return data


def read_message(sock: socket.socket) -> tuple[bytes, bytes]:
    topic_len = int.from_bytes(recv_exact(sock, 4), "big")
    topic = recv_exact(sock, topic_len)

    msg_len = int.from_bytes(recv_exact(sock, 4), "big")
    msg = recv_exact(sock, msg_len)

    return topic, msg


def send_subscription(sock: socket.socket, topics: list[str]) -> None:
    packet = len(topics).to_bytes(4, "big")

    for topic in topics:
        topic_bytes = topic.encode("utf-8")
        packet += len(topic_bytes).to_bytes(4, "big")
        packet += topic_bytes

    sock.sendall(packet)


def main() -> None:
    host = "127.0.0.1"
    port = 8889

    if len(sys.argv) < 2:
        print("usage: python3 subscriber.py <topic1> [topic2] ... [--port PORT]")
        return

    args = sys.argv[1:]
    topics = []
    i = 0

    while i < len(args):
        if args[i] == "--port":
            port = int(args[i + 1])
            i += 2
        else:
            topics.append(args[i])
            i += 1

    if not topics:
        print("please provide at least one topic")
        return

    sock = socket.create_connection((host, port))
    print(f"connected to subscriber port {port}")

    send_subscription(sock, topics)
    print(f"subscribed to: {topics}")

    try:
        while True:
            topic, msg = read_message(sock)

            try:
                topic_text = topic.decode("utf-8")
            except UnicodeDecodeError:
                topic_text = str(topic)

            try:
                msg_text = msg.decode("utf-8")
            except UnicodeDecodeError:
                msg_text = str(msg)

            print(f"received topic={topic_text!r} message={msg_text!r}")
    except ConnectionError:
        print("subscriber connection closed")
    finally:
        sock.close()


if __name__ == "__main__":
    main()