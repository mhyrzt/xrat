#!/usr/bin/env python3
import argparse
import base64
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Optional
from urllib.parse import parse_qs, unquote, urlparse
from urllib.request import urlopen


@dataclass
class Node:
    protocol: str
    address: str
    port: int
    uuid: Optional[str] = None
    password: Optional[str] = None
    method: Optional[str] = None
    network: str = "tcp"
    tls: Optional[str] = None
    sni: Optional[str] = None
    host: Optional[str] = None
    path: Optional[str] = None
    name: Optional[str] = None

    def dedup_key(self) -> tuple:
        return (self.protocol, self.address, self.port, self.uuid, self.password)


def b64_decode_text(data: str) -> str:
    """Decode base64 with automatic padding fix."""
    padding = len(data) % 4
    if padding:
        data += "=" * (4 - padding)
    return base64.urlsafe_b64decode(data).decode("utf-8", errors="ignore")


def decode_b64_bytes(data: bytes) -> str:
    return base64.b64decode(data).decode("utf-8", errors="ignore")


def decode_or_json_text(data: bytes) -> str:
    try:
        decoded_text = decode_b64_bytes(data).strip()
    except Exception:
        decoded_text = ""

    if decoded_text:
        return decoded_text

    raw_text = data.decode("utf-8", errors="ignore").strip()
    if not raw_text:
        raise ValueError("Response body is empty")

    try:
        json.loads(raw_text)
    except json.JSONDecodeError as exc:
        raise ValueError("Response is neither valid base64 nor valid JSON") from exc

    return raw_text


def parse_vless(line: str) -> Node:
    parsed = urlparse(line.replace("vless://", "https://"))
    if not parsed.hostname or not parsed.port:
        raise ValueError("Missing address or port")

    qs = parse_qs(parsed.query)
    return Node(
        protocol="vless",
        address=parsed.hostname,
        port=parsed.port,
        uuid=parsed.username or None,
        network=qs.get("type", ["tcp"])[0],
        tls=qs.get("security", [None])[0],
        sni=qs.get("sni", [None])[0],
        host=qs.get("host", [None])[0],
        path=unquote(qs.get("path", [""])[0]) or None,
        name=unquote(parsed.fragment) if parsed.fragment else None,
    )


def parse_vmess(line: str) -> Node:
    payload = line.replace("vmess://", "")
    data = json.loads(b64_decode_text(payload))
    if not data.get("add") or not data.get("port"):
        raise ValueError("Missing required address/port fields in vmess JSON")

    return Node(
        protocol="vmess",
        address=data.get("add"),
        port=int(data.get("port")),
        uuid=data.get("id"),
        network=data.get("net", "tcp"),
        tls=data.get("tls"),
        sni=data.get("sni"),
        host=data.get("host"),
        path=data.get("path"),
        name=data.get("ps"),
    )


def parse_ss(line: str) -> Node:
    parsed = urlparse(line.replace("ss://", "https://"))
    if not parsed.hostname or not parsed.port:
        raise ValueError("Missing address or port")
    if not parsed.username:
        raise ValueError("Missing base64 userinfo")

    userinfo = b64_decode_text(parsed.username)
    if ":" not in userinfo:
        raise ValueError("Invalid Shadowsocks userinfo format")

    method, password = userinfo.split(":", 1)
    return Node(
        protocol="ss",
        address=parsed.hostname,
        port=parsed.port,
        method=method,
        password=password,
        name=unquote(parsed.fragment) if parsed.fragment else None,
    )


def normalize(node: Node) -> None:
    if not node.network:
        node.network = "tcp"
    if node.network == "ws":
        if not node.host and node.sni:
            node.host = node.sni
        if not node.path:
            node.path = "/"
    if node.network == "grpc" and not node.path:
        node.path = "/"
    if node.tls == "":
        node.tls = None


def parse_line(line: str) -> Optional[Node]:
    line = line.strip()
    if not line or line.startswith("#"):
        return None

    try:
        if line.startswith("vless://"):
            return parse_vless(line)
        if line.startswith("vmess://"):
            return parse_vmess(line)
        if line.startswith("ss://"):
            return parse_ss(line)
        return None
    except Exception as exc:
        print(
            f"[ERROR] Failed to parse line: {line[:80]} ... Reason: {exc}",
            file=sys.stderr,
        )
        return None


def parse_text(config_text: str) -> list[dict]:
    nodes = []
    seen = set()

    for line in config_text.splitlines():
        node = parse_line(line)
        if not node:
            continue

        normalize(node)
        key = node.dedup_key()
        if key in seen:
            continue

        seen.add(key)
        nodes.append(asdict(node))

    return nodes


def fetch_url(url: str) -> bytes:
    with urlopen(url) as response:
        return response.read()


def save_nodes(output_path: Path, nodes: list[dict]) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8") as target:
        json.dump(nodes, target, indent=2, ensure_ascii=False)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Download a base64 subscription URL, decode it, and parse configs into JSON."
    )
    parser.add_argument("url", help="Subscription URL to download")
    parser.add_argument("output_file", type=Path, help="JSON file to write parsed configs to")
    return parser


def main() -> int:
    args = build_parser().parse_args()
    try:
        encoded_data = fetch_url(args.url)
        config_text = decode_or_json_text(encoded_data)

        try:
            parsed_json = json.loads(config_text)
        except json.JSONDecodeError:
            nodes = parse_text(config_text)
            save_nodes(args.output_file, nodes)
            print(
                f"Decoded subscription and saved {len(nodes)} parsed nodes to: "
                f"{args.output_file}"
            )
            return 0

        save_nodes(args.output_file, parsed_json)
    except Exception as exc:
        print(f"Error: {exc}", file=sys.stderr)
        return 1

    print(f"Saved raw JSON config directly to: {args.output_file}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
