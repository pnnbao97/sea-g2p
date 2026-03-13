import json
import struct
import os
def build_binary_dict():
    # Cấu hình đường dẫn (sử dụng đường dẫn tương đối để linh hoạt hơn)
    base_dir = os.path.dirname(os.path.abspath(__file__))
    common_json_path = os.path.join(base_dir, "common.json")
    sea_json_path = os.path.join(base_dir, "sea.json")
    output_bin_path = os.path.join(base_dir, "src", "sea_g2p", "phone_dict", "phone_dict.bin")
    # Đảm bảo thư mục đầu ra tồn tại
    os.makedirs(os.path.dirname(output_bin_path), exist_ok=True)
    print(f"--- Bắt đầu build Phone Dict Binary ---")
    # 1. Load dữ liệu từ JSON
    print(f"Đang đọc {common_json_path}...")
    with open(common_json_path, 'r', encoding='utf-8') as f:
        common_raw = json.load(f)
    
    print(f"Đang đọc {sea_json_path} (có thể mất vài giây)...")
    with open(sea_json_path, 'r', encoding='utf-8') as f:
        sea_raw = json.load(f)
    # 2. Chuẩn hóa dữ liệu
    common_data = []
    for word, phones in common_raw.items():
        common_data.append((
            word.lower(), 
            phones.get('vi', ''), 
            phones.get('en', '')
        ))
    merged_data = []
    for word, phone in sea_raw.items():
        merged_data.append((word.lower(), phone))
    # 3. Sắp xếp để hỗ trợ Binary Search sau này
    print("Đang sắp xếp dữ liệu...")
    common_data.sort()
    merged_data.sort()
    # 4. Tạo String Pool (Tối ưu kích thước file bằng cách không lưu trùng lặp chuỗi)
    print("Đang tạo String Pool...")
    all_strings = set()
    for w, vi, en in common_data:
        all_strings.add(w)
        all_strings.add(vi)
        all_strings.add(en)
    for w, p in merged_data:
        all_strings.add(w)
        all_strings.add(p)
    
    string_list = sorted(list(all_strings))
    string_to_id = {s: i for i, s in enumerate(string_list)}
    
    print(f"Thống kê: {len(merged_data)} từ merged, {len(common_data)} từ common, {len(string_list)} chuỗi duy nhất.")
    # 5. Ghi file Binary (Định dạng SEAP v1)
    with open(output_bin_path, 'wb') as f:
        # Header: Magic(4), Version(4), StringCount(4), MergedCount(4), CommonCount(4)
        f.write(b'SEAP') # Magic
        f.write(struct.pack('<I', 1)) # Version 1
        f.write(struct.pack('<I', len(string_list)))
        f.write(struct.pack('<I', len(merged_data)))
        f.write(struct.pack('<I', len(common_data)))
        
        # Placeholder cho Offsets (StringOffsetsPos, MergedPos, CommonPos)
        pos_header_offsets = f.tell()
        f.write(struct.pack('<III', 0, 0, 0))
        # A. Ghi dữ liệu chuỗi (String Data)
        string_data_start = f.tell()
        string_offsets = []
        for s in string_list:
            string_offsets.append(f.tell() - string_data_start)
            f.write(s.encode('utf-8'))
            f.write(b'\0') # Null terminator
        
        # B. Ghi bảng Offset của chuỗi
        string_offsets_pos = f.tell()
        for off in string_offsets:
            f.write(struct.pack('<I', off))
        
        # C. Ghi bảng Merged (WordID, PhoneID)
        merged_pos = f.tell()
        for w, p in merged_data:
            f.write(struct.pack('<II', string_to_id[w], string_to_id[p]))
            
        # D. Ghi bảng Common (WordID, ViPhoneID, EnPhoneID)
        common_pos = f.tell()
        for w, v, e in common_data:
            f.write(struct.pack('<III', string_to_id[w], string_to_id[v], string_to_id[e]))
            
        # Cập nhật các vị trí Offset vào Header
        f.seek(pos_header_offsets)
        f.write(struct.pack('<III', string_offsets_pos, merged_pos, common_pos))
        
    print(f"Thành công! File binary đã được lưu tại: {output_bin_path}")
if __name__ == "__main__":
    build_binary_dict()