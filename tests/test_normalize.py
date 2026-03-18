import pytest
from sea_g2p import Normalizer

@pytest.fixture
def normalizer():
    return Normalizer()

# Combined test cases from multiple categories
TEST_CASES = [
    # ─── 1. SỐ THÔNG THƯỜNG ────────────────────────────────────────────────────
    ("0",        "không"),
    ("1",        "một"),
    ("10",       "mười"),
    ("11",       "mười một"),
    ("21",       "hai mươi mốt"),
    ("100",      "một trăm"),
    ("1000",     "một nghìn"),
    ("1001",     "một nghìn không trăm lẻ một"),
    ("1005",     "một nghìn không trăm lẻ năm"),
    ("115",      "một trăm mười lăm"),
    ("45.565",   "bốn mươi lăm nghìn năm trăm sáu mươi lăm"),
    ("45.005",   "bốn mươi lăm nghìn không trăm lẻ năm"),
    ("1000000",  "một triệu"),
    ("1234567",  "một triệu hai trăm ba mươi bốn nghìn năm trăm sáu mươi bảy"),

    # ─── 2. SỐ THẬP PHÂN / SỐ CÓ DẤU PHÂN CÁCH ──────────────────────────────
    ("1.000",    "một nghìn"),
    ("1.000.000","một triệu"),
    ("3,14",     "ba phẩy một bốn"),
    ("1.3",      "một chấm ba"),
    ("1.25",     "một chấm hai lăm"),
    ("1.5",      "một chấm năm"),
    ("1.55",     "một chấm năm lăm"),

    # ─── 3. SỐ ĐIỆN THOẠI ────────────────────────────────────────────────────
    ("0912345678",    "không chín một hai ba bốn năm sáu bảy tám"),
    ("+84912345678",  "cộng tám bốn chín một hai ba bốn năm sáu bảy tám"),

    # ─── 4. SỐ THỨ TỰ ─────────────────────────────────────────────────────────
    ("thứ 1",  "thứ nhất"),
    ("thứ 4",  "thứ tư"),
    ("thứ 5",  "thứ năm"),
    ("hạng 1", "hạng nhất"),
    ("thứ 10",  "thứ mười"),
    ("Thứ 10",  "thứ mười"),

    # ─── 5. PHÉP NHÂN ─────────────────────────────────────────────────────────
    ("3 x 4",  "ba nhân bốn"),
    ("10 x 20","mười nhân hai mươi"),

    # ─── 6. NGÀY THÁNG ─────────────────────────────────────────────────────────
    ("21/02/2025", "ngày hai mươi mốt tháng hai năm hai nghìn không trăm hai mươi lăm"),
    ("01-01-2024", "ngày một tháng một năm hai nghìn không trăm hai mươi bốn"),
    ("31.12.2023", "ngày ba mươi mốt tháng mười hai năm hai nghìn không trăm hai mươi ba"),
    ("31.12.1997", "ngày ba mươi mốt tháng mười hai năm một nghìn chín trăm chín mươi bảy"),
    ("21/02", "ngày hai mươi mốt tháng hai"),
    ("01/12", "ngày một tháng mười hai"),
    ("02/2025", "tháng hai năm hai nghìn không trăm hai mươi lăm"),
    ("12/2024", "tháng mười hai năm hai nghìn không trăm hai mươi bốn"),
    ("32/01", "ba mươi hai trên không một"),
    ("01/13", "không một trên mười ba"),
    ("tháng 3/2026", "tháng ba năm hai nghìn không trăm hai mươi sáu"),

    # ─── 7. THỜI GIAN ─────────────────────────────────────────────────────────
    ("14h30",   "mười bốn giờ ba mươi phút"),
    ("8h05",    "tám giờ không năm phút"),
    ("0h00",    "không giờ không phút"),
    ("23:59",   "hai mươi ba giờ năm mươi chín phút"),
    ("12:00:00","mười hai giờ không phút không giây"),
    ("10:20 phút", "mười giờ hai mươi phút"),
    ("12:00:00 giây", "mười hai giờ không phút không giây"),

    # ─── 8. TIỀN TỆ ──────────────────────────────────────────────────────────
    ("100$",   "một trăm <en>u s d</en>"),
    ("$50",    "năm mươi <en>u s d</en>"),
    ("200 USD","hai trăm <en>u s d</en>"),
    ("500 VND","năm trăm việt nam đồng"),
    ("50 euro","năm mươi <en>euro</en>"),
    ("1000đ",  "một nghìn đồng"),
    ("75%",    "bảy mươi lăm phần trăm"),
    ("15,4% xuống còn 8,3%", "mười lăm phẩy bốn phần trăm xuống còn tám phẩy ba phần trăm"),
    ("370 tỷ USD", "ba trăm bảy mươi tỷ <en>u s d</en>"),
    ("5 triệu VND", "năm triệu việt nam đồng"),
    ("10 nghìn USD", "mười nghìn <en>u s d</en>"),
    ("8,92 tỷ USD", "tám phẩy chín hai tỷ <en>u s d</en>"),

    # ─── 9. ĐƠN VỊ ĐO LƯỜNG ─────────────────────────────────────────────────
    ("50km",  "năm mươi ki lô mét"),
    ("100m",  "một trăm mét"),
    ("30cm",  "ba mươi xen ti mét"),
    ("5mm",   "năm mi li mét"),
    ("75kg",  "bảy mươi lăm ki lô gam"),
    ("500g",  "năm trăm gam"),
    ("250ml", "hai trăm năm mươi mi li lít"),
    ("2l",    "hai lít"),
    ("10ha",  "mười héc ta"),
    ("50m2",  "năm mươi mét vuông"),
    ("20m3",  "hai mươi mét khối"),
    ("300.000km", "ba trăm nghìn ki lô mét"),
    ("5 triệu km", "năm triệu ki lô mét"),
    ("1,5 ha", "một phẩy năm héc ta"),
    ("1.5 ha", "một chấm năm héc ta"),
    ("AN/ASQ", "<en>a n</en> trên <en>a s q</en>"),

    # ─── 10. KHOẢNG / DÃY SỐ ──────────────────────────────────────────────────
    ("700-900", "bảy trăm đến chín trăm"),
    ("0,5-0,9", "không phẩy năm đến không phẩy chín"),

    # ─── 11. SỐ LA MÃ ────────────────────────────────────────────────────────
    ("Thế kỷ XXI",  "thế kỷ hai mươi mốt"),
    ("Chương IV",   "chương bốn"),
    ("Hồi IX",      "hồi chín"),
    ("Phần III",    "phần ba"),
    ("Thế kỷ XX",   "thế kỷ hai mươi"),

    # ─── 12. CHỮ CÁI ─────────────────────────────────────────────────────────
    ("ký tự A",     "ký tự a"),
    ("chữ B",       "chữ bê"),
    ("ký tự 'C'",   "ký tự xê"),
    ("chữ cái Z",   "chữ cái dét"),
    ("kí tự w",     "kí tự đắp liu"),

    # ─── 13. TỪ VIẾT TẮT TIẾNG VIỆT ─────────────────────────────────────────
    ("UBND",  "uỷ ban nhân dân"),
    ("TP.HCM","thành phố hồ chí minh"),
    ("TPHCM", "thành phố hồ chí minh"),
    ("CSGT",  "cảnh sát giao thông"),
    ("LHQ",   "liên hợp quốc"),
    ("CLB",   "câu lạc bộ"),
    ("HLV",   "huấn luyện viên"),
    ("TS",    "tiến sĩ"),
    ("GS",    "giáo sư"),
    ("THPT",  "trung học phổ thông"),
    ("THCS",  "trung học cơ sở"),

    # ─── 14. THẺ TIẾNG ANH (EN TAG) ─────────────────────────────────────────
    ("<en>Hello</en>",                              "<en>Hello</en>"),
    ("<en>Hello 123</en>",                          "<en>Hello 123</en>"),
    ("Xin chào <en>Good morning</en>",              "xin chào <en>Good morning</en>"),
    ("Ngày 21/02 <en>February 21</en>",             "ngày hai mươi mốt tháng hai <en>February 21</en>"),
    ("<en>AI</en> là trí tuệ nhân tạo",             "<en>AI</en> là trí tuệ nhân tạo"),

    # ─── 15. DẤU CÂU ─────────────────────────────────────────────────────────
    ("A & B",              "a và bê"),
    ("A + B",              "a cộng bê"),
    ("A = B",              "a bằng bê"),
    ("#1",                 "thăng một"),
    ("(text in brackets)", "text in brackets"),
    ("[text in brackets]", "text in brackets"),
    ("(giờ Mỹ)", "giờ mỹ"),
    ("hiệu lực từ 0h01 (giờ Mỹ), trong vòng", "hiệu lực từ không giờ không một phút, giờ mỹ, trong vòng"),
    ("hiệu lực từ 0h01 (giờ Mỹ) trong vòng", "hiệu lực từ không giờ không một phút, giờ mỹ, trong vòng"),
    ("kết thúc (0h01).", "kết thúc, không giờ không một phút."),
    ("chỉ số là 7,05 - đường huyết là 1.8", "chỉ số là bảy phẩy không năm, đường huyết là một chấm tám"),
    ("ta có !hôm nay thật kì lạ; ta sẽ đi,chơi", "ta có! hôm nay thật kì lạ, ta sẽ đi, chơi"),

    # ─── 16. CẤU TRÚC VĂN BẢN ──────────────────────────────────────────────
    ("Đoạn 1.\nĐoạn 2.", "đoạn một.\nđoạn hai."),
    ("\n12 tiêm kích", "\nmười hai tiêm kích"),

    # ─── 17. VIẾT TẮT ĐƠN GIẢN ──────────────────────────────────────────────
    ("v.v",  "vân vân"),
    ("v/v",  "về việc"),
    ("đ/c",  "địa chỉ"),

    # ─── 18. TRƯỜNG HỢP HỖN HỢP ──────────────────────────────────────────────
    ("Ngày 21/02/2025 lúc 14h30, giá vàng đạt 100$ tại TPHCM",
     "ngày hai mươi mốt tháng hai năm hai nghìn không trăm hai mươi lăm lúc mười bốn giờ ba mươi phút, giá vàng đạt một trăm <en>u s d</en> tại thành phố hồ chí minh"),
    ("Thế kỷ XXI chứng kiến sự phát triển của <en>AI</en> và vũ trụ học",
     "thế kỷ hai mươi mốt chứng kiến sự phát triển của <en>AI</en> và vũ trụ học"),
    ("Đề án 06 và Chỉ thị 04", "đề án không sáu và chỉ thị không bốn"),

    # ─── 19. AN TOÀN (KHÔNG NHẦM LẪN) ──────────────────────────────────────────
    ("Anh M. đi bộ", "anh mờ đi bộ"),
    ("Vitamin G", "vitamin gờ"),
    ("L là tên riêng", "lờ là tên riêng"),
    ("5m chiều dài", "năm mét chiều dài"),
    ("Đơn vị km", "đơn vị ki lô mét"),

    # ─── 20. EMAIL ───────────────────────────────────────────────────────────
    ("Liên hệ qua email pnnbao@gmail.com nhé.", "liên hệ qua email <en>pnnbao</en> a còng <en>gmail</en> chấm com nhé."),
    ("Email: contact@example.com", "email, <en>contact</en> a còng <en>example</en> chấm com"),

    # ─── 21. VIẾT TẮT ALPHANUMERIC (ENGLISH STYLE) ──────────────────────────
    ("Mô hình B2B rất phổ biến.", "mô hình <en>b two b</en> rất phổ biến."),
    ("Tôi dùng camera K3.", "tôi dùng camera ca ba."),
    ("Mã số A1B.", "mã số a một bê."),
    ("Tôi đang học về AI.", "tôi đang học về <en>a i</en>."),
    ("Dự án VYE.", "dự án <en>v y e</en>."),

    # ─── 22. PHÂN BIỆT HOA THƯỜNG TRONG CÂU ─────────────────────────────────
    ("TÔI ĐI HỌC", "tôi đi học"),
    ("Chào mừng bạn đến với CTY.", "chào mừng bạn đến với <en>c t y</en>."),

    # ─── 23. TOÀN DIỆN (CẢI TIẾN MỚI) ──────────────────────────────────────────
    # URLs
    ("Truy cập https://vieneu.io để biết thêm chi tiết.", "truy cập <en>https</en> <en>vieneu</en> chấm <en>i o</en> để biết thêm chi tiết."),
    ("Website www.google.com rất hữu ích.", "website <en>www</en> chấm <en>google</en> chấm com rất hữu ích."),
    ("Trang web chính thức là https://openai.com.", "trang web chính thức là <en>https</en> <en>openai</en> chấm com."),
    ("Tài liệu nằm ở www.example.org/docs.", "tài liệu nằm ở <en>www</en> chấm <en>example</en> chấm o rờ gờ gạch <en>docs</en>."),

    # Slashes / Địa chỉ
    ("Địa chỉ nhà tôi là 123/4 đường Nguyễn Trãi.", "địa chỉ nhà tôi là một trăm hai mươi ba trên bốn đường nguyễn trãi."),
    ("Tỷ lệ là 100/2.", "tỷ lệ là một trăm trên hai."),

    # Ký hiệu toán học
    ("Nếu x > 5 và y ≤ 10 thì xấp xỉ ≈ 0.", "nếu ích lớn hơn năm và y nhỏ hơn hoặc bằng mười thì xấp xỉ xấp xỉ không."),
    ("Nhiệt độ là 30°C ± 2°C.", "nhiệt độ là ba mươi độ xê cộng trừ hai độ xê."),
    ("Biểu thức ≥ 10.", "biểu thức lớn hơn hoặc bằng mười."),

    # Đơn vị đo lường mở rộng
    ("Dung lượng 16GB.", "dung lượng mười sáu <en>gigabyte</en>."),
    ("File nặng 50MB.", "file nặng năm mươi <en>megabyte</en>."),
    ("Ổ cứng 1TB.", "ổ cứng một <en>terabyte</en>."),
    ("Căn hộ 75sqm.", "căn hộ bảy mươi lăm mét vuông."),
    ("Bể bơi 100cum.", "bể bơi một trăm mét khối."),
    ("Âm thanh 80db.", "âm thanh tám mươi <en>decibel</en>."),
    ("Trọng lượng 10lb.", "trọng lượng mười <en>pound</en>."),
    ("Màn hình 24in.", "màn hình hai mươi bốn <en>inch</en>."),
    ("Độ phân giải 300dpi.", "độ phân giải ba trăm <en>d p i</en>."),
    ("Độ pH của nước là 7.", "độ pê hát của nước là bảy."),
    ("3.46 USD/gallon", "ba chấm bốn sáu <en>u s d</en> trên <en>gallon</en>"),

    # Emails mở rộng
    ("Email công việc: admin@fpt.vn", "email công việc, <en>admin</en> a còng <en>f p t</en> chấm <en>v n</en>"),
    ("Liên hệ hotmail: test@hotmail.com", "liên hệ hotmail, <en>test</en> a còng <en>hotmail</en> chấm com"),
    ("Liên hệ qua email pnnbao@proton.com nhé.", "liên hệ qua email <en>pnnbao</en> a còng <en>proton</en> chấm com nhé."),

    # Redundant expansion (symbol + unit)
    ("#1kg", "thăng một ki lô gam"),

    # ─── 24. CÂU TEST THỰC TẾ ──────────────────────────────────────────────────
    ("Ông Lưu Trung Thái, Chủ tịch HĐQT MB cho biết, vốn hóa của ngân hàng đã tăng gần 10 lần kể từ năm 2017, đạt khoảng 8,5 tỷ USD, tạo nền tảng cho mục tiêu 10 tỷ USD vào năm 2027.",
     "ông lưu trung thái, chủ tịch hội đồng quản trị <en>m b</en> cho biết, vốn hóa của ngân hàng đã tăng gần mười lần kể từ năm hai nghìn không trăm mười bảy, đạt khoảng tám phẩy năm tỷ <en>u s d</en>, tạo nền tảng cho mục tiêu mười tỷ <en>u s d</en> vào năm hai nghìn không trăm hai mươi bảy."),

    # ─── 25. THÊM MỚI (CẢI THIỆN) ──────────────────────────────────────────
    ("Số tiền là 1.000.000.000.000 đồng.", "số tiền là một nghìn tỷ đồng."),
    ("“Lời chào cao hơn mâm cỗ”", "lời chào cao hơn mâm cỗ"),
    ("‘Trân trọng’", "trân trọng"),
    ("Chào <en>world</en> xinh đẹp", "chào <en>world</en> xinh đẹp"),
    ("Nếu đã từng đọc cuốn sách trên của Simon, hoặc đã xem anh thuyết trình về khái niệm tại sao trên diễn đàn TED.com, thì có lẽ bạn không còn xa lạ với vòng tròn vàng.", "nếu đã từng đọc cuốn sách trên của simon, hoặc đã xem anh thuyết trình về khái niệm tại sao trên diễn đàn <en>t e d</en> chấm com, thì có lẽ bạn không còn xa lạ với vòng tròn vàng."),

    # ─── 26. TRƯỜNG HỢP CẠNH (EDGE CASES) ─────────────────────────────────────
    ("MixedCase Acronyms như ChatGPT hay Claude.", "mixed case acronyms như chat <en>g p t</en> hay claude."),
    ("Unit mix: 10km/h và 5m/s.", "unit mix, mười ki lô mét trên giờ và năm mét trên giây."),
    ("Số cực lớn: 1.000.000.000.000.000.000", "số cực lớn, một tỷ tỷ"),
    ("Email với tên miền lạ: user@domain.tech", "email với tên miền lạ, <en>user</en> a còng <en>domain</en> chấm <en>tech</en>"),
    
    # ─── 27. PHẨY VÀ DẤU NHÁY (PRIMES & APOSTROPHES) ──────────────────────────
    ("A' là một ký tự đặc biệt", "a phẩy là một ký tự đặc biệt"),
    ("Đây là phút 1'", "đây là phút một phẩy"),
    ("I don't know why", "i don't know why"),
    ("It's a beautiful day", "it's a beautiful day"),
    ("Giá của 'Sản phẩm' này là $10", "giá của sản phẩm này là mười <en>u s d</en>"),
    ("Giá SP500 hôm nay là 4.200,5 điểm", "giá ét pê năm trăm hôm nay là bốn nghìn hai trăm phẩy năm điểm"),
    ("Trường hợp 1,23,4", "trường hợp một phẩy hai ba phẩy bốn"),
    ("Trường hợp 1,234,567", "trường hợp một triệu hai trăm ba mươi bốn nghìn năm trăm sáu mươi bảy"),

    # ─── 28. CÁC TRƯỜNG HỢP MỚI (SCIENTIFIC, CURRENCY, ENGLISH SEPARATORS) ───
    ("3.2e5 km", "ba chấm hai nhân mười mũ năm ki lô mét"),
    ("6.626e-34", "sáu chấm sáu hai sáu nhân mười mũ trừ ba mươi bốn"),
    ("1.5×10^-3", "một chấm năm nhân mười mũ trừ ba"),
    ("Tôi mua nó với giá $1,299.99.", "tôi mua nó với giá một nghìn hai trăm chín mươi chín phẩy chín chín <en>u s d</en>."),
    ("€3,50", "ba phẩy năm <en>euro</en>"),
    ("¥120000", "một trăm hai mươi nghìn yên"),
    ("1Gbps", "một <en>gigabits per second</en>"),
    ("1,299,495", "một triệu hai trăm chín mươi chín nghìn bốn trăm chín mươi lăm"),
    ("1,299", "một phẩy hai chín chín"),
    ("1,299.5", "một nghìn hai trăm chín mươi chín phẩy năm"),
    ("Số tiền là 17.200 VNĐ", "số tiền là mười bảy nghìn hai trăm việt nam đồng"),
    ("Giá cổ phiếu tăng từ $0.000045 lên $1,234.5678 trong 3.5×10^6 giao dịch.", "giá cổ phiếu tăng từ không chấm không không không không bốn lăm <en>u s d</en> lên một nghìn hai trăm ba mươi bốn phẩy năm sáu bảy tám <en>u s d</en> trong ba chấm năm nhân mười mũ sáu giao dịch."),
    # ─── 29. URL, REPO, EMAIL (EXTENDED) ────────────────────────────────
    ("Trang chủ là https://openai.com.", "trang chủ là <en>https</en> <en>openai</en> chấm com."),
    ("Repo nằm ở github.com/user/project.", "repo nằm ở <en>github</en> chấm com gạch <en>user</en> gạch <en>project</en>."),
    ("Tài liệu đọc tại docs.python.org.", "tài liệu đọc tại <en>docs</en> chấm <en>python</en> chấm o rờ gờ."),
    ("Hãy gửi email đến support@example.com.", "hãy gửi email đến <en>support</en> a còng <en>example</en> chấm com."),
    ("Địa chỉ IP là 192.168.1.1 hoặc 10.0.0.1.", "địa chỉ <en>i p</en> là một chín hai chấm một sáu tám chấm một chấm một hoặc một không chấm không chấm không chấm một."),
    ("Phiên bản phần mềm là 1.25.3.4", "phiên bản phần mềm là một chấm hai năm chấm ba chấm bốn"),
    ("IPv6 là 2001:0db8:85a3:0000:0000:8a2e:0370:7334", "<en>i p v</en> sáu là hai không không một hai chấm không <en>d b</en> tám hai chấm tám năm <en>a</en> ba hai chấm không không không không hai chấm không không không không hai chấm tám <en>a</en> hai <en>e</en> hai chấm không ba bảy không hai chấm bảy ba ba bốn"),
    ("Số 123.456.789", "số một trăm hai mươi ba triệu bốn trăm năm mươi sáu nghìn bảy trăm tám mươi chín"),
    ("Mã này là 192.16.2", "mã này là một chín hai chấm một sáu chấm hai"),
    ("File tải tại ftp://example.org/data.zip.", "file tải tại <en>f t p</en> <en>example</en> chấm o rờ gờ gạch <en>data</en> chấm <en>zip</en>."),
    ("Nhiệt độ ngoài trời là -3.5°C.", "nhiệt độ ngoài trời là âm ba chấm năm độ xê."),
    ("Anh ấy chạy 10.000m trong 27:45.", "anh ấy chạy mười nghìn mét trong hai mươi bảy phút bốn mươi lăm giây."),
    ("Tỉ số USD/EUR đang tăng.", "tỉ số <en>u s d</en> trên <en>e u r</en> đang tăng."),
    ("Anh ta kiếm được ¥120000 mỗi tháng.", "anh ta kiếm được một trăm hai mươi nghìn yên mỗi tháng."),
    ("Giá là $50 cho mỗi sản phẩm.", "giá là năm mươi <en>u s d</en> cho mỗi sản phẩm."),
    ("Phí dịch vụ là €10 mỗi người.", "phí dịch vụ là mười <en>euro</en> mỗi người."),
    ("Nó nặng 10lb cho mỗi bao.", "nó nặng mười <en>pound</en> cho mỗi bao."),
    ("Giá là £5 mỗi cái.", "giá là năm <en>pound</en> mỗi cái."),
    ("Thưởng là ₩1000 cho bạn.", "thưởng là một nghìn won cho bạn."),
    ("Hằng số Avogadro là 6.022e23 mol^-1.", "hằng số avogadro là sáu chấm không hai hai nhân mười mũ hai mươi ba mol mũ trừ một."),
    ("Giới hạn lim(x→0) sin(x)/x = 1.", "giới hạn lim, ích đến không, sin, ích, trên ích bằng một."),
    ("Ông ấy là PGS.TS ngành AI.", "ông ấy là phó giáo sư tiến sĩ ngành <en>a i</en>."),
    ("Dữ liệu dạng JSON.", "dữ liệu dạng <en>j son</en>."),
    ("Du lịch tại UAE.", "du lịch tại u a e."),
    ("Chỉ số VN-Index giảm.", "chỉ số <en>v n</en> index giảm."),
    ("Hệ điều hành MS DOS.", "hệ điều hành <en>m s dos</en>."),
    ("Dùng MI5 và MI6.", "dùng <en>m i five</en> và <en>m i six</en>."),
    ("Bảo mật 2FA.", "bảo mật <en>two f a</en>."),
    ("Máy tính TX-0.", "máy tính <en>t x zero</en>."),
    ("File backup nằm ở /home/user/data_v3.2.tar.gz.", "file backup nằm ở gạch <en>home</en> gạch <en>user</en> gạch <en>data</en> gạch dưới <en>v</en> ba chấm hai chấm <en>tar</en> chấm <en>g z</en>."),
    ("Log lỗi ghi tại error_log_2024-10-21.txt.", "log lỗi ghi tại <en>error</en> gạch dưới <en>log</en> gạch dưới hai không hai bốn gạch ngang một không gạch ngang hai một chấm <en>txt</en>."),

    # ─── 30. EXTREME CASES ─────────────────────────────────────
    ("Dân số thế giới khoảng 7,888,000,000 người (~7.9B).",
     "dân số thế giới khoảng bảy tỷ tám trăm tám mươi tám triệu người, khoảng bảy chấm chín tỷ."),
    ("Latency trung bình chỉ ~42ms / request qua REST API.",
     "latency trung bình chỉ khoảng bốn mươi hai mi li giây trên request qua <en>rest</en> <en>a p i</en>."),
    ("Liên hệ qua email research.ai+test@example-domain.org.",
     "liên hệ qua email <en>research</en> chấm <en>ai</en> cộng <en>test</en> a còng <en>example</en> gạch ngang <en>domain</en> chấm o rờ gờ."),
    ("Gửi báo cáo đến admin_v2@server.ai.",
     "gửi báo cáo đến <en>admin</en> gạch dưới <en>v</en> hai a còng <en>server</en> chấm <en>ai</en>."),
    ("Repo nằm ở https://github.com/user/project-v2.",
     "repo nằm ở <en>https</en> <en>github</en> chấm com gạch <en>user</en> gạch <en>project</en> gạch ngang <en>v</en> hai."),
    ("Tại sao khi x 8 thì nó lại là 8x - mà với lại 20 x 8 = ?", "tại sao khi ích tám thì nó lại là tám ích, mà với lại hai mươi nhân tám bằng?"),

    # ─── 31. KÝ HIỆU - GIỮA SỐ CÓ ĐỘ LỚN KHÁC NHAU ────────────────────────────
    # Dấu - giữa 2 số có số chữ số chênh lệch > 1 → không phải range → giữ nguyên
    ("RAM hệ thống là 128GB DDR5-6400.",
     "<en>ram</en> hệ thống là một trăm hai mươi tám <en>gigabyte</en> đê đê rờ năm sáu nghìn bốn trăm."),
    ("CPU Core i9-14900K chạy ở xung nhịp 6,0 GHz nhưng nhiệt độ lên tới 95°C.", "<en>c p u</en> core i chín mười bốn nghìn chín trăm ca chạy ở xung nhịp sáu gi ga héc nhưng nhiệt độ lên tới chín mươi lăm độ xê."),

    # ─── 32. TECHNICAL REGRESSION TESTS ──────────────────────────────────────
    ("Đường dẫn Windows: C:\\Users\\dev\\data\\report_2026-03-11.log.", "đường dẫn windows, <en>c</en> hai chấm gạch <en>users</en> gạch <en>dev</en> gạch <en>data</en> gạch <en>report</en> gạch dưới hai không hai sáu gạch ngang không ba gạch ngang một một chấm <en>log</en>."),
    ("Chuỗi có placeholder ___PROTECTED_EN_TAG_0___ để kiểm tra xung đột.", "chuỗi có placeholder protected en tag không để kiểm tra xung đột."),
    ("Username của tôi là user_2024_dev.", "username của tôi là <en>user</en> gạch dưới hai không hai bốn gạch dưới <en>dev</en>."),
    ("Gửi báo cáo đến admin_v2@server.ai.", "gửi báo cáo đến <en>admin</en> gạch dưới <en>v</en> hai a còng <en>server</en> chấm <en>ai</en>."),
    ("Máy chủ dự phòng là http://127.0.0.1:5000/health.", "máy chủ dự phòng là <en>http</en> một hai bảy chấm không chấm không chấm một hai chấm năm không không không gạch <en>health</en>."),
    ("Thiết bị mã TX-0 vẫn còn trong bảo tàng.", "thiết bị mã <en>t x zero</en> vẫn còn trong bảo tàng."),
    ("Dataset gồm 3.2M samples (~1.8TB audio).", "dataset gồm ba chấm hai triệu samples, khoảng một chấm tám <en>terabyte</en> audio."),
    ("GPU NVIDIA RTX 4090 có 24GB GDDR6X VRAM.", "<en>g p u</en> <en>n v d a</en> <en>r t x</en> bốn nghìn không trăm chín mươi có hai mươi bốn <en>gigabyte</en> gờ đê đê rờ sáu ích <en>v ram</en>."),
    ("API local chạy ở http://localhost:8080/api/v2?lang=vi#top.", "<en>a p i</en> local chạy ở <en>http</en> <en>localhost</en> hai chấm tám không tám không gạch <en>api</en> gạch <en>v</en> hai hỏi <en>lang</en> bằng <en>v i</en> thăng <en>top</en>."),
    ("Tỷ lệ P/E là 28.7x.", "tỷ lệ pê trên e là hai mươi tám chấm bảy ích."),
    ("Câu lệnh SQL: SELECT * FROM users WHERE id=1;", "câu lệnh <en>s q l</en>, <en>select</en> sao <en>from</en> users <en>where</en> id bằng một"),
    ("EPS quý này đạt $3.45.Tiếng Việt có dấu: Hoà, Hòa, Hòa.", "<en>e p s</en> quý này đạt ba chấm bốn lăm <en>u s d</en>. tiếng việt có dấu, hoà, hòa, hòa."),

    # ─── 33. USER REPORTED EDGE CASES ──────────────────────────────────────────
    ("Nhiệt độ là -5°C", "nhiệt độ là âm năm độ xê"),
    ("Tọa độ (-2.5;0)", "tọa độ, âm hai chấm năm, không"),
    ("Họp lúc 8g sáng", "họp lúc tám giờ sáng"),
    ("090-123-4567", "không chín không một hai ba bốn năm sáu bảy"),
    ("mã ISO 9001:2015", "mã <en>i s o</en> chín nghìn không trăm lẻ một hai chấm hai nghìn không trăm mười lăm"),
    ("giải khối rubik 4x4x4 ngắn nhất.", "giải khối rubik bốn nhân bốn nhân bốn ngắn nhất."),
    ("hình chữ nhật 3x4", "hình chữ nhật ba nhân bốn"),
    ("màn hình 1920x1080", "màn hình một nghìn chín trăm hai mươi nhân một nghìn không trăm tám mươi"),
    ("kích thước 10 x 20 x 30", "kích thước mười nhân hai mươi nhân ba mươi"),
    ("kích thước 10 x 20 x 30 x 40", "kích thước mười nhân hai mươi nhân ba mươi nhân bốn mươi"),
    ("kích thước 10 x 20 x 30 x 40 x 50", "kích thước mười nhân hai mươi nhân ba mươi nhân bốn mươi nhân năm mươi"),
    ("phòng họp 5m x 10m", "phòng họp năm mét nhân mười mét"),
    ("diện tích 5×10 m2", "diện tích năm nhân mười mét vuông"),
    ("tôi đang ở Washington D.C", "tôi đang ở <en>washington d c</en>"),
    ("tôi đang ở Washington DC", "tôi đang ở <en>washington d c</en>"),

    # ─── 34. NGÀY THÁNG VÀ PHÂN SỐ (HEURISTIC) ─────────────────────────────
    ("Tôi sinh vào 3/5", "tôi sinh vào ngày ba tháng năm"),
    ("Xác suất là 3/5", "xác suất là ba trên năm"),
    ("1/2 + 1/3 = 5/6", "một trên hai cộng một trên ba bằng năm trên sáu"),
    ("Ngày 3/5 tôi tính 1/2 + 1/2", "ngày ba tháng năm tôi tính một trên hai cộng một trên hai"),
    ("Tỉ lệ là 1/4", "tỉ lệ là một trên bốn"),
    ("Sinh nhật là 20/10", "sinh nhật là ngày hai mươi tháng mười"),
    ("Lễ 2/9", "lễ ngày hai tháng chín"),
    ("Tết 1/1", "tết ngày một tháng một"),
    ("lấy 0.5/0.9 x 3 = ?", "lấy không chấm năm trên không chấm chín nhân ba bằng?"),
    ("khoản 3 điều 45 nghị định 12/2021/NĐ-CP . 45/8000", "khoản ba điều bốn mươi lăm nghị định tháng mười hai năm hai nghìn không trăm hai mươi mốt trên nờ đê <en>c p</en>. bốn mươi lăm trên tám nghìn"),

    # ─── 35. PHÂN BIỆT 123/4 (ADDRESS VS OTHER) ───────────────────────────
    ("Nhà tôi ở số 123/4 đường Nguyễn Trãi.", "nhà tôi ở số một trăm hai mươi ba trên bốn đường nguyễn trãi."),
    ("Hẻm 123/4 này rất chật.", "hẻm một trăm hai mươi ba trên bốn này rất chật."),
    ("Ngõ 123/4 là nhà bác tôi.", "ngõ một trăm hai mươi ba trên bốn là nhà bác tôi."),
    ("Số nhà 123/4.", "số nhà một trăm hai mươi ba trên bốn."),
    ("Giá trị là 123/4.", "giá trị là một trăm hai mươi ba trên bốn."),
    ("Tỷ lệ 123/4.", "tỷ lệ một trăm hai mươi ba trên bốn."),
    ("Kết quả 123/4.", "kết quả một trăm hai mươi ba trên bốn."),
    ("Vào ngày 20/10/2024, gia đình tôi đã quyết định tổ chức một buổi tiệc nhỏ", "vào ngày hai mươi tháng mười năm hai nghìn không trăm hai mươi bốn, gia đình tôi đã quyết định tổ chức một buổi tiệc nhỏ"),
    ("Lợi nhuận đạt 1.25B USD trong Q4/2025 (+12.75%).", "lợi nhuận đạt một chấm hai lăm tỷ <en>u s d</en> trong quý bốn hai không hai lăm, cộng mười hai chấm bảy lăm phần trăm."),
    ("Hóa chất: NaCl, NaOH, HCl, HClO, NaClO, ZnO, CuO, FeO, HCN, HF, NaCN, NaBr, KI, KOH, KCl, KBr, MgO", "hóa chất, nờ a xê lờ, nờ a ô hát, hát xê lờ, hát xê lờ ô, nờ a xê lờ ô, dét nờ ô, xê u ô, ép e ô, hát xê nờ, hát ép, nờ a xê nờ, nờ a bê rờ, ca i, ca ô hát, ca xê lờ, ca bê rờ, mờ gờ ô"),
    ("CH3COOH + NaOH → CH3COONa", "xê hát ba xê ô ô hát cộng nờ a ô hát đến xê hát ba xê ô ô nờ a"),
    ("Phản ứng: 2H2 + O2 → 2H2O", "phản ứng, hai hát hai cộng ô hai đến hai hát hai ô"),
    
    # ─── 36. TECHNICAL CAMELCASE SPLITTING ──────────────────────────────────
    ("getUserIDFromDB", "get user <en>i d</en> from <en>d b</en>"),
    ("getUserById", "get user by id"),
    ("setVolumeLevel", "set volume level"),
    ("iPhone", "i Phone"),
    ("HClO", "hát xê lờ ô"),
    ("ChatGPT", "chat <en>g p t</en>"),
    ("tôi đang đi du lịch Đà Lạt với người yêu cũ...", "tôi đang đi du lịch Đà Lạt với người yêu cũ."),
    ("CO2", "xê ô hai"),
    ("CO₂", "xê ô hai")
    ]

@pytest.mark.parametrize("input_text, expected", TEST_CASES)
def test_normalize(normalizer, input_text, expected):
    actual = normalizer.normalize(input_text)
    # Clean up whitespace for comparison
    actual_clean = " ".join(actual.split()).lower()
    expected_clean = " ".join(expected.split()).lower()
    
    # Special handling for brackets/punctuation to match expectation
    # Note: brackets are sometimes kept or removed depending on normalizer state
    # We follow the user provided expected strings
    assert actual_clean == expected_clean
