import os
import csv

base_dir = r"C:/Users/86188/Desktop/QR2Pic-main/scripts/QR_0811"
mapping_file = r"C:/Users/86188/Desktop/QR2Pic-main/scripts/QR_0811_mapping.csv"

uuid_to_name = {}
with open(mapping_file, "r", encoding="utf-8") as f:
    reader = csv.DictReader(f)
    for row in reader:
        image_id = row["image_id"]
        original_name = row["original_name"]
        uuid_to_name[image_id] = original_name

print(f"已加载 {len(uuid_to_name)} 条映射记录")

subfolders = [
    d for d in os.listdir(base_dir) if os.path.isdir(os.path.join(base_dir, d))
]
print(f"找到 {len(subfolders)} 个子文件夹")

total_renamed = 0
total_skipped = 0

for subfolder in subfolders:
    subfolder_path = os.path.join(base_dir, subfolder)
    files = [f for f in os.listdir(subfolder_path) if f.endswith("_qr.png")]

    print(f"\n处理文件夹: {subfolder} ({len(files)} 个文件)")

    for filename in files:
        uuid = filename.replace("_qr.png", "")

        if uuid in uuid_to_name:
            original_name = uuid_to_name[uuid]
            new_name = original_name.replace(".jpg", "") + "_qr.png"

            old_path = os.path.join(subfolder_path, filename)
            new_path = os.path.join(subfolder_path, new_name)

            if os.path.exists(new_path):
                print(f"  跳过: {filename} -> {new_name} (目标文件已存在)")
                total_skipped += 1
            else:
                os.rename(old_path, new_path)
                print(f"  重命名: {filename} -> {new_name}")
                total_renamed += 1
        else:
            print(f"  跳过: {filename} (映射不存在)")
            total_skipped += 1

print(f"\n完成! 共重命名 {total_renamed} 个文件，跳过 {total_skipped} 个文件")
