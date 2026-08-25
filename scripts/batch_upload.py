#!/usr/bin/env python3
"""
批量图片上传及二维码生成脚本
用于将图片批量上传到服务器，生成对应的二维码，并保存映射关系
"""

import os
import sys
import json
import csv
import time
import hashlib
import argparse
from pathlib import Path
from datetime import datetime
from typing import Optional
from urllib.parse import urlparse

try:
    import requests
except ImportError:
    print("请安装依赖: pip install requests qrcode[pil] pillow")
    sys.exit(1)

try:
    import qrcode
except ImportError:
    print("请安装依赖: pip install qrcode[pil] pillow")
    sys.exit(1)


ALLOWED_EXTENSIONS = {".jpg", ".jpeg", ".png", ".gif", ".webp"}
DEFAULT_API_URL = "http://localhost:3000/upload"
DEFAULT_INPUT_DIR = "./input_images"
DEFAULT_OUTPUT_DIR = "./output_qr_codes"
DEFAULT_MAPPING_FILE = "./mapping.csv"


def calculate_file_hash(file_path: str) -> str:
    """计算文件的 MD5 哈希值用于去重"""
    hash_md5 = hashlib.md5()
    with open(file_path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            hash_md5.update(chunk)
    return hash_md5.hexdigest()


def is_allowed_file(filename: str) -> bool:
    """检查文件扩展名是否允许"""
    ext = Path(filename).suffix.lower()
    return ext in ALLOWED_EXTENSIONS


def get_file_extension(filename: str) -> str:
    """获取文件扩展名"""
    ext = Path(filename).suffix.lower()
    return ext.lstrip(".")


def upload_image(api_url: str, file_path: str, retry: int = 3) -> Optional[dict]:
    """上传图片到服务器"""
    for attempt in range(retry):
        try:
            with open(file_path, "rb") as f:
                files = {"file": f}
                response = requests.post(api_url, files=files, timeout=30, verify=False)

            if response.status_code == 200:
                return response.json()
            elif response.status_code == 413:
                print(f"  [错误] 文件过大: {file_path}")
                return None
            else:
                print(f"  [错误] 上传失败 (状态码 {response.status_code}): {file_path}")
        except requests.exceptions.RequestException as e:
            print(f"  [重试 {attempt + 1}/{retry}] 网络错误: {e}")
            time.sleep(1)

    return None


def generate_qr_code(data: str, output_path: str) -> bool:
    """生成二维码图片"""
    try:
        qr = qrcode.QRCode(
            version=1,
            error_correction=qrcode.ERROR_CORRECT_L,
            box_size=10,
            border=4,
        )
        qr.add_data(data)
        qr.make(fit=True)

        img = qr.make_image(fill_color="black", back_color="white")
        img.save(output_path)
        return True
    except Exception as e:
        print(f"  [错误] 生成二维码失败: {e}")
        return False


def init_mapping_file(mapping_file: str) -> None:
    """初始化映射文件"""
    if not os.path.exists(mapping_file):
        with open(mapping_file, "w", newline="", encoding="utf-8") as f:
            writer = csv.writer(f)
            writer.writerow(
                [
                    "image_id",
                    "original_name",
                    "file_path",
                    "qr_code_path",
                    "file_hash",
                    "uploaded_at",
                ]
            )


def append_mapping(mapping_file: str, record: dict) -> None:
    """追加映射记录"""
    with open(mapping_file, "a", newline="", encoding="utf-8") as f:
        writer = csv.writer(f)
        writer.writerow(
            [
                record["image_id"],
                record["original_name"],
                record["file_path"],
                record["qr_code_path"],
                record["file_hash"],
                record["uploaded_at"],
            ]
        )


def load_existing_mappings(mapping_file: str) -> set:
    """加载已处理的图片哈希值集合"""
    if not os.path.exists(mapping_file):
        return set()

    hashes = set()
    with open(mapping_file, "r", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for row in reader:
            if "file_hash" in row and row["file_hash"]:
                hashes.add(row["file_hash"])
    return hashes


def get_all_images(input_path: Path, recursive: bool = False):
    """获取目录下所有图片文件"""
    if recursive:
        return [
            f for f in input_path.rglob("*") if f.is_file() and is_allowed_file(f.name)
        ]
    else:
        return [
            f for f in input_path.iterdir() if f.is_file() and is_allowed_file(f.name)
        ]


def process_batch(
    api_url: str,
    input_dir: str,
    output_dir: str,
    mapping_file: str,
    skip_existing: bool = True,
    base_url: Optional[str] = None,
    recursive: bool = False,
    preserve_structure: bool = False,
) -> None:
    """批量处理图片"""
    input_path = Path(input_dir)
    output_path = Path(output_dir)

    if not input_path.exists():
        print(f"错误: 输入目录不存在 - {input_dir}")
        sys.exit(1)

    output_path.mkdir(parents=True, exist_ok=True)
    init_mapping_file(mapping_file)

    existing_hashes = load_existing_mappings(mapping_file) if skip_existing else set()

    image_files = get_all_images(input_path, recursive)
    image_files.sort()

    if not image_files:
        print(f"未找到支持格式的图片文件: {ALLOWED_EXTENSIONS}")
        return

    print(f"找到 {len(image_files)} 个图片文件")
    print(f"输出目录: {output_dir}")
    print(f"映射文件: {mapping_file}")
    print("-" * 50)

    success_count = 0
    skip_count = 0
    error_count = 0

    for idx, image_file in enumerate(image_files, 1):
        print(f"[{idx}/{len(image_files)}] 处理: {image_file.name}")

        file_hash = calculate_file_hash(str(image_file))

        if skip_existing and file_hash in existing_hashes:
            print(f"  [跳过] 文件已存在")
            skip_count += 1
            continue

        result = upload_image(api_url, str(image_file))

        if result is None:
            print(f"  [错误] 上传失败")
            error_count += 1
            continue

        image_id = result.get("id")
        image_url = result.get("url")

        qr_data = image_id
        if base_url:
            qr_data = f"{base_url.rstrip('/')}/view/{image_id}"

        qr_filename = f"{image_id}_qr.png"

        if preserve_structure and recursive:
            rel_path = image_file.relative_to(input_path)
            qr_output_path = output_path / rel_path.parent / qr_filename
            qr_output_path.parent.mkdir(parents=True, exist_ok=True)
        else:
            qr_output_path = output_path / qr_filename

        qr_path = qr_output_path

        if qr_data and generate_qr_code(qr_data, str(qr_path)):
            record = {
                "image_id": image_id,
                "original_name": image_file.name,
                "file_path": image_url,
                "qr_code_path": str(qr_path),
                "file_hash": file_hash,
                "uploaded_at": datetime.now().isoformat(),
            }
            append_mapping(mapping_file, record)
            existing_hashes.add(file_hash)

            print(f"  [成功] ID: {image_id}")
            print(f"  [成功] URL: {image_url}")
            print(f"  [成功] 二维码: {qr_filename}")
            success_count += 1
        else:
            print(f"  [错误] 生成二维码失败")
            error_count += 1

    print("-" * 50)
    print(f"处理完成: 成功 {success_count}, 跳过 {skip_count}, 失败 {error_count}")


def main():
    parser = argparse.ArgumentParser(
        description="批量图片上传及二维码生成工具",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
示例:
  python batch_upload.py --api-url http://localhost:3000/upload --input ./photos
  python batch_upload.py --input ./photos --output ./qrcodes --mapping ./records.csv
  python batch_upload.py --api-url https://your-domain.com/upload --base-url https://your-domain.com --no-skip
        """,
    )

    parser.add_argument(
        "--api-url",
        type=str,
        default=os.environ.get("API_URL", DEFAULT_API_URL),
        help=f"上传API地址 (默认: {DEFAULT_API_URL})",
    )
    parser.add_argument(
        "--input",
        type=str,
        default=os.environ.get("INPUT_DIR", DEFAULT_INPUT_DIR),
        help=f"输入图片目录 (默认: {DEFAULT_INPUT_DIR})",
    )
    parser.add_argument(
        "--output",
        type=str,
        default=os.environ.get("OUTPUT_DIR", DEFAULT_OUTPUT_DIR),
        help=f"二维码输出目录 (默认: {DEFAULT_OUTPUT_DIR})",
    )
    parser.add_argument(
        "--mapping",
        type=str,
        default=os.environ.get("MAPPING_FILE", DEFAULT_MAPPING_FILE),
        help=f"映射记录文件 (默认: {DEFAULT_MAPPING_FILE})",
    )
    parser.add_argument(
        "--base-url",
        type=str,
        default=os.environ.get("BASE_URL", ""),
        help="用于生成二维码的访问基础URL (可选)",
    )
    parser.add_argument("--no-skip", action="store_true", help="不跳过已处理的文件")
    parser.add_argument(
        "--retry", type=int, default=3, help="上传失败重试次数 (默认: 3)"
    )
    parser.add_argument("-r", "--recursive", action="store_true", help="递归处理子目录")
    parser.add_argument(
        "--preserve-structure",
        action="store_true",
        help="保留目录结构 (与 --recursive 一起使用)",
    )

    args = parser.parse_args()

    print("=" * 50)
    print("批量图片上传及二维码生成工具")
    print("=" * 50)
    print(f"API地址: {args.api_url}")
    print(f"输入目录: {args.input}")
    print(f"输出目录: {args.output}")
    print(f"映射文件: {args.mapping}")
    if args.base_url:
        print(f"基础URL: {args.base_url}")
    print("=" * 50)

    process_batch(
        api_url=args.api_url,
        input_dir=args.input,
        output_dir=args.output,
        mapping_file=args.mapping,
        skip_existing=not args.no_skip,
        base_url=args.base_url if args.base_url else None,
        recursive=args.recursive,
        preserve_structure=args.preserve_structure,
    )


if __name__ == "__main__":
    main()
