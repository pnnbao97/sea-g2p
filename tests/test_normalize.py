import pytest
from sea_g2p import Normalizer


@pytest.fixture
def normalizer():
    return Normalizer()


# Bộ test chuẩn hóa, gom theo nhóm chủ đề. Mỗi cặp (input, expected) là một
# "regression guard"; expected được so khớp sau khi gộp khoảng trắng + lowercase.
TEST_CASES = [
    # ══ 1. SỐ NGUYÊN & SỐ LỚN ════════════════════════════════════════════════
    ("0", "không"),
    ("1", "một"),
    ("10", "mười"),
    ("11", "mười một"),
    ("21", "hai mươi mốt"),
    ("100", "một trăm"),
    ("115", "một trăm mười lăm"),
    ("1000", "một nghìn"),
    ("1001", "một nghìn không trăm lẻ một"),
    ("1005", "một nghìn không trăm lẻ năm"),
    ("1,000,000", "một triệu"),
    # Số nguyên >= 7 chữ số viết liền -> đọc rời từng số (heuristic mã/ID).
    ("1234567", "một hai ba bốn năm sáu bảy"),
    ("798237423", "bảy chín tám hai ba bảy bốn hai ba"),
    ("-1234567", "âm một hai ba bốn năm sáu bảy"),
    ("Số cực lớn: 1.000.000.000.000.000.000", "số cực lớn, một tỷ tỷ"),
    ("Số tiền là 1.000.000.000.000 đồng.", "số tiền là một nghìn tỷ đồng."),
    ("Số 123.456.789", "số một trăm hai mươi ba triệu bốn trăm năm mươi sáu nghìn bảy trăm tám mươi chín"),
    ("Mã mẫu có dạng 000123 và cần đọc đúng.", "mã mẫu có dạng không không không một hai ba và cần đọc đúng."),

    # ══ 2. SỐ THẬP PHÂN & DẤU PHÂN CÁCH ══════════════════════════════════════
    ("1.000", "một nghìn"),
    ("1.000.000", "một triệu"),
    ("45.565", "bốn mươi lăm nghìn năm trăm sáu mươi lăm"),
    ("45.005", "bốn mươi lăm nghìn không trăm lẻ năm"),
    ("3,14", "ba phẩy một bốn"),
    ("1.3", "một chấm ba"),
    ("1.25", "một chấm hai lăm"),
    ("1.5", "một chấm năm"),
    ("1.55", "một chấm năm lăm"),
    ("9.1", "chín chấm một"),
    ("9,1", "chín phẩy một"),
    ("1,299", "một phẩy hai chín chín"),
    ("1,299.5", "một nghìn hai trăm chín mươi chín phẩy năm"),
    ("1,299,495", "một triệu hai trăm chín mươi chín nghìn bốn trăm chín mươi lăm"),
    ("Trường hợp 1,23,4", "trường hợp một phẩy hai ba phẩy bốn"),
    ("Trường hợp 1,234,567", "trường hợp một triệu hai trăm ba mươi bốn nghìn năm trăm sáu mươi bảy"),
    ("Tổng hợp gồm 1,000.00 USD và 1.000,00 EUR.", "tổng hợp gồm một nghìn <en>u s d</en> và một nghìn <en>euro</en>."),

    # ══ 3. KÝ HIỆU KHOA HỌC & LŨY THỪA ═══════════════════════════════════════
    ("3.2e5 km", "ba chấm hai nhân mười mũ năm ki lô mét"),
    ("6.626e-34", "sáu chấm sáu hai sáu nhân mười mũ trừ ba mươi bốn"),
    ("1.5×10^-3", "một chấm năm nhân mười mũ trừ ba"),
    ("Sai số đo được là -1.5e-3 đơn vị.", "sai số đo được là âm một chấm năm nhân mười mũ trừ ba đơn vị."),
    ("Hằng số Avogadro là 6.022e23 mol^-1.", "hằng số avogadro là sáu chấm không hai hai nhân mười mũ hai mươi ba mol mũ trừ một."),
    ("Phản ứng có nồng độ 10^-3 mol/L.", "phản ứng có nồng độ mười mũ trừ ba mol trên lít."),

    # ══ 4. SỐ THỨ TỰ & PHÉP NHÂN ═════════════════════════════════════════════
    ("thứ 1", "thứ nhất"),
    ("thứ 4", "thứ tư"),
    ("thứ 5", "thứ năm"),
    ("thứ 10", "thứ mười"),
    ("Thứ 10", "thứ mười"),
    ("hạng 1", "hạng nhất"),
    ("3 x 4", "ba nhân bốn"),
    ("10 x 20", "mười nhân hai mươi"),
    ("hình chữ nhật 3x4", "hình chữ nhật ba nhân bốn"),
    ("giải khối rubik 4x4x4 ngắn nhất.", "giải khối rubik bốn nhân bốn nhân bốn ngắn nhất."),
    ("màn hình 1920x1080", "màn hình một nghìn chín trăm hai mươi nhân một nghìn không trăm tám mươi"),
    ("phòng họp 5m x 10m", "phòng họp năm mét nhân mười mét"),
    ("diện tích 5×10 m2", "diện tích năm nhân mười mét vuông"),

    # ══ 5. NGÀY THÁNG ════════════════════════════════════════════════════════
    ("21/02/2025", "ngày hai mươi mốt tháng hai năm hai nghìn không trăm hai mươi lăm"),
    ("01-01-2024", "ngày một tháng một năm hai nghìn không trăm hai mươi bốn"),
    ("31.12.2023", "ngày ba mươi mốt tháng mười hai năm hai nghìn không trăm hai mươi ba"),
    ("31.12.1997", "ngày ba mươi mốt tháng mười hai năm một nghìn chín trăm chín mươi bảy"),
    ("21/02", "ngày hai mươi mốt tháng hai"),
    ("01/12", "ngày một tháng mười hai"),
    ("02/2025", "tháng hai năm hai nghìn không trăm hai mươi lăm"),
    ("12/2024", "tháng mười hai năm hai nghìn không trăm hai mươi bốn"),
    ("tháng 3/2026", "tháng ba năm hai nghìn không trăm hai mươi sáu"),
    ("quý 1/2025 tăng", "quý một năm hai nghìn không trăm hai mươi lăm tăng"),
    ("quý 4/2024", "quý bốn năm hai nghìn không trăm hai mươi bốn"),
    # Từ láy KHÔNG bị gộp: chỉ gộp "ngày ngày"/"năm năm"/"tháng tháng" khi từ
    # ngay sau là chữ số (tức do pass ngày-tháng sinh ra).
    ("ngày ngày vẫn đông khách", "ngày ngày vẫn đông khách"),
    ("ngày ngày năn nỉ anh Nam", "ngày ngày năn nỉ anh nam"),
    ("suốt năm năm trời anh vẫn đợi", "suốt năm năm trời anh vẫn đợi"),
    ("tháng tháng đóng tiền đều đặn", "tháng tháng đóng tiền đều đặn"),
    ("cuộc họp vào ngày 15/3", "cuộc họp vào ngày mười lăm tháng ba"),
    # "hôm ngày"/"mùng ngày" cũng chỉ gộp khi sau là chữ số.
    ("hôm ngày lễ đó cả nhà đi chơi", "hôm ngày lễ đó cả nhà đi chơi"),
    ("cúng vào mùng ngày rằm", "cúng vào mùng ngày rằm"),
    ("hôm 15/3 cả nhà đi chơi", "hôm mười lăm tháng ba cả nhà đi chơi"),
    # "âm" viết sẵn + số mang dấu trừ -> chỉ đọc MỘT "âm"...
    ("nhiệt độ âm -5 độ C", "nhiệt độ âm năm độ xê"),
    ("kết quả là âm -2", "kết quả là âm hai"),
    # ...nhưng "âm âm" thật (không phải số) giữ nguyên.
    ("giá trị âm âm là dương", "giá trị âm âm là dương"),
    ("điện tích âm và dương", "điện tích âm và dương"),
    ("mùng 5/5 là Tết Đoan Ngọ", "mùng năm tháng năm là tết đoan ngọ"),
    # Từ dẫn ngày đứng NGAY TRƯỚC -> "d/m" là ngày tháng...
    ("triều cường chiều 17/10 tràn qua", "triều cường chiều ngày mười bảy tháng mười tràn qua"),
    ("nợ này phải trả trước 30/4", "nợ này phải trả trước ngày ba mươi tháng tư"),
    # ...nhưng cách một từ thì vẫn là phân số ("chiều dài 3/4 mét").
    ("chiều dài 3/4 mét là vừa", "chiều dài ba trên bốn mét là vừa"),
    ("chị chỉ cần đổ 3/4 cốc nước", "chị chỉ cần đổ ba trên bốn cốc nước"),
    ("phân số 7/8 lớn hơn 3/4", "phân số bảy trên tám lớn hơn ba trên bốn"),
    # Khoảng ngày: vế SAU cũng thành ngày tháng.
    ("hồ sơ nhận từ 1/8 đến hết 31/8",
     "hồ sơ nhận từ ngày một tháng tám đến hết ngày ba mươi mốt tháng tám"),
    ("chạy từ 20/11 đến 25/11",
     "chạy từ ngày hai mươi tháng mười một đến ngày hai mươi lăm tháng mười một"),
    # ∆ (U+2206) = delta toán học, khác Δ Hy Lạp.
    ("nhiệt lượng Q = mc∆t", "nhiệt lượng qui bằng mờ xê đen ta tê"),
    ("định luật Húc F = k∆l", "định luật húc ép bằng ca đen ta lờ"),
    # Bộ/sở dạng "&" bổ sung.
    ("nộp về phòng KH&ĐT", "nộp về phòng kế hoạch đầu tư"),
    ("chuyên viên Sở TT&TT", "chuyên viên sở thông tin truyền thông"),
    # TLD mới + "@" trong URL.
    ("link forms.gle/xnk27 nhé", "link forms chấm gle gạch chéo ích nờ ca hai bảy nhé"),
    ("kênh youtube.com/@toanthayvu", "kênh youtube chấm com gạch chéo a còng toan thay vu"),
    # Biển số xe: chạy trước pass giờ để "51H" không thành "năm mươi mốt giờ".
    ("biển số 51H-123.45 vượt đèn đỏ",
     "biển số năm mươi mốt hát một hai ba chấm bốn năm vượt đèn đỏ"),
    ("taxi biển 30K-567.89 trả lại ví",
     "taxi biển ba mươi ca năm sáu bảy chấm tám chín trả lại ví"),
    # Biển số seri có chữ số + đuôi 4 số (xe máy) và seri chứa X (không thành "nhân").
    ("xe máy biển 52N5-1234", "xe máy biển năm mươi hai nờ năm một hai ba bốn"),
    ("biển 59X1-123.45", "biển năm mươi chín ích một một hai ba chấm bốn năm"),
    # Biển số CỤT cần từ dẫn "biển/BKS"; "51h" trần vẫn là thời lượng.
    ("xe biển số 51H đi qua trạm", "xe biển số năm mươi mốt hát đi qua trạm"),
    ("làm việc 51h mỗi tuần", "làm việc năm mươi mốt giờ mỗi tuần"),
    # "ML/AI" không phải cặp đơn vị -> đọc acronym, KHÔNG đọc "mi li lít".
    ("kỹ sư AI/ML lương cao", "kỹ sư <en>a i</en> trên <en>m l</en> lương cao"),
    ("nền tảng Core ML/AI", "nền tảng core <en>m l</en> trên <en>a i</en>"),
    ("tốc độ 120 km/h", "tốc độ một trăm hai mươi ki lô mét trên giờ"),
    ("chỉ số P/E cao", "chỉ số phê trên e cao"),
    # Mã chữ-số: phần số ≥3 chữ số đọc từng chữ số như đọc mã.
    ("mã vé ABC-1234", "mã vé <en>a b c</en> một hai ba bốn"),
    # ...nhưng ≤2 chữ số vẫn đọc số đếm (COVID-19, U-23).
    ("bệnh nhân COVID-19", "bệnh nhân <en>covid</en> mười chín"),
    ("đội U-23 Việt Nam", "đội u hai mươi ba việt nam"),
    # "#" + mã số: bỏ "thăng", đọc từng chữ số.
    ("đơn hàng #45021 đã rời kho", "đơn hàng bốn năm không hai một đã rời kho"),
    # Số tổng đài đọc từng chữ số.
    ("tổng đài 1900 thu phí", "tổng đài một chín không không thu phí"),
    # Đơn vị y tế mmol/l.
    ("đường huyết 6,2 mmol/l", "đường huyết sáu phẩy hai mi li mol trên lít"),
    # Ngày không hợp lệ -> đọc dãy số "trên", không vỡ cú pháp.
    ("32/01", "ba mươi hai trên không một"),
    ("01/13", "không một trên mười ba"),
    ("ngày 30/2/2024", "ngày ba mươi trên hai trên hai nghìn không trăm hai mươi bốn"),
    ("ngày 31/4/2023", "ngày ba mươi mốt trên bốn trên hai nghìn không trăm hai mươi ba"),
    # Heuristic ngày-tháng vs phân số (cùng dạng "a/b").
    ("Tôi sinh vào 3/5", "tôi sinh vào ngày ba tháng năm"),
    ("Xác suất là 3/5", "xác suất là ba trên năm"),
    ("Tỉ lệ là 1/4", "tỉ lệ là một trên bốn"),
    ("Sinh nhật là 20/10", "sinh nhật là ngày hai mươi tháng mười"),
    ("Lễ 2/9", "lễ ngày hai tháng chín"),
    ("Tết 1/1", "tết ngày một tháng một"),
    # Ngữ cảnh ngày mở rộng (phiên/mùng/mồng).
    ("phiên 15/3", "phiên ngày mười lăm tháng ba"),
    ("mùng 5/5 âm lịch", "mùng năm tháng năm âm lịch"),
    ("mồng 3/3", "mồng ba tháng ba"),
    ("Ngày 3/5 tôi tính 1/2 + 1/2", "ngày ba tháng năm tôi tính một trên hai cộng một trên hai"),
    ("Vào ngày 20/10/2024, gia đình tôi đã quyết định tổ chức một buổi tiệc nhỏ", "vào ngày hai mươi tháng mười năm hai nghìn không trăm hai mươi bốn, gia đình tôi đã quyết định tổ chức một buổi tiệc nhỏ"),
    ("khoản 3 điều 45 nghị định 12/2021/NĐ-CP . 45/8000", "khoản ba điều bốn mươi lăm nghị định tháng mười hai năm hai nghìn không trăm hai mươi mốt trên nờ đê xê phê. bốn mươi lăm trên tám nghìn"),
    ("Log lỗi: ERROR[2025-03-11T14:22:03Z].", "log lỗi, <en>error</en>, ngày mười một tháng ba năm hai nghìn không trăm hai mươi lăm tê mười bốn giờ hai mươi hai phút ba giây dét."),

    # ══ 6. THỜI GIAN ═════════════════════════════════════════════════════════
    ("14h30", "mười bốn giờ ba mươi phút"),
    ("08h05", "tám giờ năm phút"),
    ("0h00", "không giờ không phút"),
    ("23:59", "hai mươi ba giờ năm mươi chín phút"),
    ("12:00:00", "mười hai giờ không phút không giây"),
    ("10:20 phút", "mười giờ hai mươi phút"),
    ("12:00:00 giây", "mười hai giờ không phút không giây"),
    ("Họp lúc 8g sáng", "họp lúc tám giờ sáng"),
    ("Anh ấy chạy 10.000m trong 27:45.", "anh ấy chạy mười nghìn mét trong hai mươi bảy phút bốn mươi lăm giây."),

    # ══ 7. SỐ ĐIỆN THOẠI / HOTLINE / SỐ BÀN ══════════════════════════════════
    ("0912345678", "không chín một hai ba bốn năm sáu bảy tám"),
    ("+84912345678", "cộng tám bốn chín một hai ba bốn năm sáu bảy tám"),
    ("gọi 1900 1234", "gọi một chín không không một hai ba bốn"),
    ("tổng đài 1800.6601", "tổng đài một tám không không sáu sáu không một"),
    ("1900-1080", "một chín không không một không tám không"),
    ("1800-1900", "một tám không không một chín không không"),
    ("năm 1900 có", "năm một nghìn chín trăm có"),  # 1900 đứng một mình vẫn là năm
    ("(028) 3822 1234", "không hai tám, ba tám hai hai, một hai ba bốn"),
    ("024 3822 1234", "không hai bốn, ba tám hai hai, một hai ba bốn"),
    ("+84 28 3822 1234", "cộng tám bốn, hai tám, ba tám hai hai, một hai ba bốn"),
    ("090-123-4567", "không chín không, một hai ba, bốn năm sáu bảy"),
    ("Số điện thoại: (+84) 901-234-567.", "số điện thoại, cộng tám mươi bốn, chín không một, hai ba bốn, năm sáu bảy."),
    ("Số điện thoại: 0921 978 951 là số của Phạm Nguyễn Ngọc Bảo", "số điện thoại, không chín hai một, chín bảy tám, chín năm một là số của phạm nguyễn ngọc bảo"),
    ("Số thẻ tín dụng: 4111-2222-3333-4444 (Visa).", "số thẻ tín dụng, bốn một một một, hai hai hai hai, ba ba ba ba, bốn bốn bốn bốn, visa."),
    ("Mã số thuế cá nhân: 8123456789-001 (Vui lòng đọc từng số).", "mã số thuế cá nhân, tám một hai ba bốn năm sáu bảy tám chín không không một, vui lòng đọc từng số."),
    ("Mã số thuế của doanh nghiệp là 0123456789-001.", "mã số thuế của doanh nghiệp là không một hai ba bốn năm sáu bảy tám chín không không một."),

    # ══ 8. TIỀN TỆ & PHẦN TRĂM ═══════════════════════════════════════════════
    ("100$", "một trăm <en>u s d</en>"),
    ("$50", "năm mươi <en>u s d</en>"),
    ("200 USD", "hai trăm <en>u s d</en>"),
    ("500 VND", "năm trăm việt nam đồng"),
    ("50 euro", "năm mươi <en>euro</en>"),
    ("1000đ", "một nghìn đồng"),
    ("Số tiền là 17.200 VNĐ", "số tiền là mười bảy nghìn hai trăm việt nam đồng"),
    ("75%", "bảy mươi lăm phần trăm"),
    ("15,4% xuống còn 8,3%", "mười lăm phẩy bốn phần trăm xuống còn tám phẩy ba phần trăm"),
    # Phần trăm âm.
    ("giảm -5% so với", "giảm âm năm phần trăm so với"),
    ("lãi suất -0,5%", "lãi suất âm không phẩy năm phần trăm"),
    ("-5% đến -2%", "âm năm phần trăm đến âm hai phần trăm"),
    ("370 tỷ USD", "ba trăm bảy mươi tỷ <en>u s d</en>"),
    ("5 triệu VND", "năm triệu việt nam đồng"),
    ("10 nghìn USD", "mười nghìn <en>u s d</en>"),
    ("8,92 tỷ USD", "tám phẩy chín hai tỷ <en>u s d</en>"),
    ("€3,50", "ba phẩy năm <en>euro</en>"),
    ("¥120000", "một trăm hai mươi nghìn yên"),
    ("Anh ta kiếm được ¥120000 mỗi tháng.", "anh ta kiếm được một trăm hai mươi nghìn yên mỗi tháng."),
    ("Giá là $50 cho mỗi sản phẩm.", "giá là năm mươi <en>u s d</en> cho mỗi sản phẩm."),
    ("Phí dịch vụ là €10 mỗi người.", "phí dịch vụ là mười <en>euro</en> mỗi người."),
    ("Giá là £5 mỗi cái.", "giá là năm <en>pound</en> mỗi cái."),
    ("Thưởng là ₩1000 cho bạn.", "thưởng là một nghìn won cho bạn."),
    ("Tôi mua nó với giá $1,299.99.", "tôi mua nó với giá một nghìn hai trăm chín mươi chín phẩy chín chín <en>u s d</en>."),
    ("Giá cổ phiếu tăng từ $0.000045 lên $1,234.5678 trong 3.5×10^6 giao dịch.", "giá cổ phiếu tăng từ không chấm không không không không bốn lăm <en>u s d</en> lên một nghìn hai trăm ba mươi bốn phẩy năm sáu bảy tám <en>u s d</en> trong ba chấm năm nhân mười mũ sáu giao dịch."),
    ("Lợi nhuận đạt 1.25B USD trong Q4/2025 (+12.75%).", "lợi nhuận đạt một chấm hai lăm tỷ <en>u s d</en> trong quý bốn hai không hai lăm, cộng mười hai chấm bảy lăm phần trăm."),
    # Tiền lóng k / tr.
    ("500k", "năm trăm nghìn"),
    ("1tr", "một triệu"),
    ("1tr5", "một triệu năm trăm nghìn"),
    ("15tr", "mười lăm triệu"),
    ("2tr3", "hai triệu ba trăm nghìn"),
    ("giá 250k", "giá hai trăm năm mươi nghìn"),

    # ══ 9. ĐƠN VỊ ĐO LƯỜNG ═══════════════════════════════════════════════════
    ("50km", "năm mươi ki lô mét"),
    ("100m", "một trăm mét"),
    ("30cm", "ba mươi xen ti mét"),
    ("5mm", "năm mi li mét"),
    ("75kg", "bảy mươi lăm ki lô gam"),
    ("500g", "năm trăm gam"),
    ("250ml", "hai trăm năm mươi mi li lít"),
    ("2l", "hai lít"),
    ("10ha", "mười héc ta"),
    ("50m2", "năm mươi mét vuông"),
    ("20m3", "hai mươi mét khối"),
    ("300.000km", "ba trăm nghìn ki lô mét"),
    ("5 triệu km", "năm triệu ki lô mét"),
    ("1,5 ha", "một phẩy năm héc ta"),
    ("1.5 ha", "một chấm năm héc ta"),
    ("5m chiều dài", "năm mét chiều dài"),
    ("Đơn vị km", "đơn vị ki lô mét"),
    ("Căn hộ 75sqm.", "căn hộ bảy mươi lăm mét vuông."),
    ("Bể bơi 100cum.", "bể bơi một trăm mét khối."),
    ("Trọng lượng 10lb.", "trọng lượng mười <en>pound</en>."),
    ("Màn hình 24in.", "màn hình hai mươi bốn <en>inch</en>."),
    ("Độ phân giải 300dpi.", "độ phân giải ba trăm <en>d p i</en>."),
    ("Độ pH của nước là 7.", "độ phê hát của nước là bảy."),
    ("Unit mix: 10km/h và 5m/s.", "unit mix, mười ki lô mét trên giờ và năm mét trên giây."),
    ("3.46 USD/gallon", "ba chấm bốn sáu <en>u s d</en> trên <en>gallon</en>"),
    # Dữ liệu / điện tử.
    ("Dung lượng 16GB.", "dung lượng mười sáu <en>gigabyte</en>."),
    ("File nặng 50MB.", "file nặng năm mươi <en>megabyte</en>."),
    ("Ổ cứng 1TB.", "ổ cứng một <en>terabyte</en>."),
    ("RAM 8GB", "<en>ram</en> tám <en>gigabyte</en>"),
    ("1Gbps", "một <en>gigabits per second</en>"),
    ("Âm thanh 80db.", "âm thanh tám mươi <en>decibel</en>."),
    # Đơn vị điện ghép camelCase (kWh, mAh).
    ("5 kWh", "năm ki lô oát giờ"),
    ("3 mAh", "ba mi li am pe giờ"),
    ("10 Ah", "mười am pe giờ"),
    ("100kWh", "một trăm ki lô oát giờ"),
    # Nhiệt độ.
    ("Nhiệt độ là 30°C ± 2°C.", "nhiệt độ là ba mươi độ xê cộng trừ hai độ xê."),
    ("Nhiệt độ ngoài trời là -3.5°C.", "nhiệt độ ngoài trời là âm ba chấm năm độ xê."),
    ("Nhiệt độ là -5°C", "nhiệt độ là âm năm độ xê"),

    # ══ 10. CHIỀU CAO / CÂN NẶNG KIỂU VIỆT ═══════════════════════════════════
    ("anh ấy cao 1m75", "anh ấy cao một mét bảy mươi lăm"),
    ("người mẫu 1m80", "người mẫu một mét tám mươi"),
    ("cao 1m8", "cao một mét tám"),
    ("người 1m6", "người một mét sáu"),
    ("bé 1m1", "bé một mét một"),
    ("nặng 1kg2", "nặng một ki lô gam hai"),
    ("sào 1m2", "sào một mét vuông"),       # m2 vẫn là mét vuông
    ("phòng 50m2", "phòng năm mươi mét vuông"),
    ("khối 20m3", "khối hai mươi mét khối"),
    # Chữ HOA đơn dính sau SỐ NGUYÊN mặc định là mã hiệu -> đánh vần; chỉ đọc
    # đơn vị khi có ngữ cảnh: số thập phân, từ dẫn tiền (M/B/K), vật chứa (L).
    ("vốn 5M", "vốn năm triệu"),          # "vốn" là từ dẫn tiền -> triệu
    ("thương vụ 5M USD", "thương vụ năm triệu <en>u s d</en>"),
    ("lương 20M một tháng", "lương hai mươi triệu một tháng"),
    ("video đạt 5M lượt xem", "video đạt năm triệu lượt xem"),
    ("quỹ đầu tư 2B đồng", "quỹ đầu tư hai tỷ đồng"),
    ("giá 100K một ly", "giá một trăm nghìn một ly"),
    ("chai 2L nước ngọt", "chai hai lít nước ngọt"),
    # Không có ngữ cảnh -> mã hiệu, đánh vần chữ cái.
    ("căn hộ 51M", "căn hộ năm mươi mốt mờ"),
    ("mã lô 51M-234 đã xuất kho", "mã lô năm mươi mốt mờ hai ba bốn đã xuất kho"),
    ("lô 12B nằm cuối dãy", "lô mười hai bê nằm cuối dãy"),
    ("iPhone 5S vẫn chạy tốt", "i phone năm ét vẫn chạy tốt"),
    ("mã 51K in trên tem", "mã năm mươi mốt ca in trên tem"),
    ("khối 12L của trường", "khối mười hai lờ của trường"),
    ("mã 51H bị phạt", "mã năm mươi mốt hát bị phạt"),
    ("căn hộ 51M", "căn hộ năm mươi mốt mờ"),
    ("phục vụ 24H", "phục vụ hai mươi bốn hát"),
    ("trực 24h liên tục", "trực hai mươi bốn giờ liên tục"),
    ("gói 450g đường", "gói bốn trăm năm mươi gam đường"),
    ("chuyến 14H30 hoãn", "chuyến mười bốn giờ ba mươi phút hoãn"),
    # Đơn vị oát trần + chiều cao/cân nặng đứng cuối câu.
    ("tấm pin 550 W", "tấm pin năm trăm năm mươi oát"),
    ("cao 1m75.", "cao một mét bảy mươi lăm."),
    ("nặng 3kg2.", "nặng ba ki lô gam hai."),

    # ══ 11. KHOẢNG / TỈ SỐ / PHÉP TRỪ (dấu gạch & gạch chéo) ══════════════════
    ("700-900", "bảy trăm đến chín trăm"),
    ("0,5-0,9", "không phẩy năm đến không phẩy chín"),
    ("tăng 5-7%", "tăng năm đến bảy phần trăm"),
    ("giảm 3-5%", "giảm ba đến năm phần trăm"),
    ("5-10 kg", "năm đến mười ki lô gam"),
    ("3-4 lần", "ba đến bốn lần"),
    # Ngữ cảnh từ/khoảng/trong -> "đến"; bằng/tính/kết quả -> "trừ".
    ("từ 5-10 ngày", "từ năm đến mười ngày"),
    ("khoảng 3-5 triệu đồng", "khoảng ba đến năm triệu đồng"),
    ("trong 5-10 ngày", "trong năm đến mười ngày"),
    ("nhiệt độ từ 20-25 độ", "nhiệt độ từ hai mươi đến hai mươi lăm độ"),
    ("bằng 10-3", "bằng mười trừ ba"),
    ("kết quả 10-3 nghĩa là", "kết quả mười trừ ba nghĩa là"),
    ("tính 12-4 ra", "tính mười hai trừ bốn ra"),
    # Chênh lệch số chữ số > 1 -> không phải khoảng, giữ hai số.
    ("RAM hệ thống là 128GB DDR5-6400.", "<en>ram</en> hệ thống là một trăm hai mươi tám <en>gigabyte</en> đê đê rờ năm sáu nghìn bốn trăm."),
    # Tỉ số thể thao -> đọc hai số rời.
    ("Việt Nam 2-1 Thái Lan", "việt nam hai một thái lan"),
    ("Arsenal 3-0 Chelsea", "arsenal ba không chelsea"),
    # en_ctx nới ngưỡng: 2 từ Anh thật (thuần chữ thường, có trong wordlist)
    # đủ kích hoạt câu Anh dù từ còn lại lẫn chữ số ("3D", "4K").
    ("print 3D technology", "print three d technology"),
    ("best 4K monitor", "best four k monitor"),
    # ...nhưng trong câu Việt thì "3D" vẫn đọc kiểu Việt.
    ("Công nghệ in 3D đang phát triển.", "công nghệ in ba đê đang phát triển."),
    ("thắng 2-0", "thắng hai không"),
    ("thua 0-2", "thua không hai"),
    ("hòa 1-1", "hòa một một"),
    ("tỉ số 3-1", "tỉ số ba một"),
    ("chung cuộc 4-2", "chung cuộc bốn hai"),
    # KHÔNG nhầm thành tỉ số -> vẫn là khoảng "đến".
    ("Điều 5-10 Luật Hình sự", "điều năm đến mười luật hình sự"),
    ("trang 5-10 Sách", "trang năm đến mười sách"),
    ("Tôi mua 5-10 quả", "tôi mua năm đến mười quả"),
    # Phân số toán + phép trừ có "=".
    ("1/2 - 1/3 = -1/6", "một trên hai trừ một trên ba bằng âm một trên sáu"),
    ("23 - 45 = -22", "hai mươi ba trừ bốn mươi lăm bằng âm hai mươi hai"),
    ("lấy 0.5/0.9 x 3 = ?", "lấy không chấm năm trên không chấm chín nhân ba bằng?"),
    ("Tại sao khi x 8 thì nó lại là 8x - mà với lại 20 x 8 = ?", "tại sao khi ích tám thì nó lại là tám ích, mà với lại hai mươi nhân tám bằng?"),
    # Phân số kiểu địa chỉ "123/4".
    ("Nhà tôi ở số 123/4 đường Nguyễn Trãi.", "nhà tôi ở số một trăm hai mươi ba trên bốn đường nguyễn trãi."),
    ("Giá trị là 123/4.", "giá trị là một trăm hai mươi ba trên bốn."),
    ("Tỷ lệ là 100/2.", "tỷ lệ là một trăm trên hai."),
    # Viết tắt địa chỉ P./Q./Đ. (theo sau là số).
    ("nhà ở Q.1 P.5", "nhà ở quận một phường năm"),
    ("Đ.3/2", "đường ba trên hai"),

    # ══ 12. TỈ LỆ DẤU HAI CHẤM (":") ═════════════════════════════════════════
    ("tỉ lệ 1:2:3", "tỉ lệ một trên hai trên ba"),
    ("01:02:03", "một giờ hai phút ba giây"),  # đủ 2 chữ số -> giờ
    ("tỉ lệ 3:4:5 nhé", "tỉ lệ ba trên bốn trên năm nhé"),
    ("tỉ lệ nợ/vốn là 1:2:3.", "tỉ lệ nợ trên vốn là một trên hai trên ba."),
    ("tỷ lệ 2:1.", "tỷ lệ hai trên một."),
    ("tỉ số 3:2.", "tỉ số ba trên hai."),
    ("mã ISO 9001:2015", "mã <en>iso</en> chín nghìn không trăm lẻ một, hai nghìn không trăm mười lăm"),
    ("Tỉ lệ bản đồ 1:50.000.", "tỉ lệ bản đồ một, năm mươi nghìn."),
    ("Tại thời điểm 02:01, tỉ số trận đấu là 2:1 nhưng tỉ lệ cược là 1:2.5.", "tại thời điểm hai giờ một phút, tỉ số trận đấu là hai trên một nhưng tỉ lệ cược là một, hai chấm năm."),
    ("Vào lúc 10:30, chỉ số nợ/vốn là 1.5:1.", "vào lúc mười giờ ba mươi phút, chỉ số nợ trên vốn là một chấm năm, một."),
    ("Tỷ lệ P/E là 28.7x.", "tỷ lệ phê trên e là hai mươi tám chấm bảy ích."),
    ("Tỉ số USD/EUR đang tăng.", "tỉ số <en>u s d</en> trên <en>euro</en> đang tăng."),
    ("AN/ASQ", "<en>a n</en> trên <en>a s q</en>"),
    # "91W": từ khi thêm đơn vị "w" -> "oát" (công suất "550 W"/"320W" phổ biến
    # hơn nhiều), chỉ số tốc độ lốp chấp nhận đọc "oát".
    ("Kích thước lốp xe 225/45R17 91W.", "kích thước lốp xe hai trăm hai mươi lăm trên bốn mươi lăm rờ mười bảy chín mươi mốt oát."),
    ("Kích thước lốp xe 45R17/22R5 91W.", "kích thước lốp xe bốn mươi lăm rờ mười bảy trên hai mươi hai rờ năm chín mươi mốt oát."),

    # ══ 13. SỐ LA MÃ ═════════════════════════════════════════════════════════
    ("Thế kỷ XXI", "thế kỷ hai mươi mốt"),
    ("Thế kỷ XX", "thế kỷ hai mươi"),
    ("Chương IV", "chương bốn"),
    ("Hồi IX", "hồi chín"),
    ("Phần III", "phần ba"),
    ("Đại hội XIII", "đại hội mười ba"),
    ("vua Louis XIV", "vua louis mười bốn"),
    ("thế chiến II", "thế chiến hai"),
    ("quý IV", "quý bốn"),
    ("quý III tăng", "quý ba tăng"),
    ("quý II", "quý hai"),
    ("lần II", "lần hai"),
    ("vòng III", "vòng ba"),
    ("Edward II", "edward hai"),
    ("vua William III", "vua william ba"),
    ("giáo hoàng Benedict XVI", "giáo hoàng benedict mười sáu"),
    # Từ dẫn "thứ" (xét TỪ CUỐI ngay trước cụm): "lần thứ IX" phải ra số.
    ("tái bản lần thứ IX", "tái bản lần thứ chín"),
    ("Đại chiến thế giới thứ II", "đại chiến thế giới thứ hai"),
    # Số La Mã MỘT ký tự (I/V/X) chỉ đọc thành số khi có từ dẫn.
    ("Chương V dài nhất", "chương năm dài nhất"),
    ("Quốc hội khóa X", "quốc hội khóa mười"),
    ("phần I", "phần một"),
    # Quý viết số La Mã kèm năm ("quý III/2027") phải ra "năm ...", không phải "trên".
    ("quý III/2027", "quý ba năm hai nghìn không trăm hai mươi bảy"),
    ("Quý I/2026 tăng trưởng", "quý một năm hai nghìn không trăm hai mươi sáu tăng trưởng"),
    # CHỈ nhận số La Mã viết HOA; chữ thường dễ trùng âm tiết tiếng Việt nên để nguyên.
    ("chương iv", "chương iv"),
    ("thế kỷ xxi", "thế kỷ xxi"),
    ("quý iii", "quý iii"),
    ("lần di chuyển", "lần di chuyển"),
    ("lần vi phạm", "lần vi phạm"),
    # Không có từ dẫn -> rơi vào nhánh acronym tiếng Anh.
    ("đĩa CD và đầu DVD", "đĩa <en>c d</en> và đầu <en>d v d</en>"),
    ("mã MC", "mã <en>m c</en>"),
    ("size XL và XXL", "size <en>x l</en> và <en>x x l</en>"),
    ("CIV và MIX", "<en>c i v</en> và <en>m i x</en>"),
    # Số thứ tự đề mục La Mã đầu dòng, kèm dấu "." -> đọc là số (không phải chữ cái).
    ("I. VỀ ĐỀ NGHỊ HUÂN CHƯƠNG LAO ĐỘNG",
     "một. về đề nghị huân chương lao động"),
    ("II. CƠ CẤU TỔ CHỨC", "hai. cơ cấu tổ chức"),
    ("III. Về đề nghị khen thưởng", "ba. về đề nghị khen thưởng"),
    ("IV. KẾT LUẬN", "bốn. kết luận"),
    ("X. PHỤ LỤC", "mười. phụ lục"),
    # Ký tự đơn trùng chữ viết tắt tên riêng -> KHÔNG đọc thành số (giữ tên đọc chữ cái).
    ("C. Mác là nhà triết học", "xê mác là nhà triết học"),
    ("V. Nguyễn Văn A", "vê nguyễn văn a"),
    ("M. Gorki", "mờ gorki"),

    # ══ 14. CHỮ CÁI ĐƠN & SPELL AN TOÀN ══════════════════════════════════════
    ("ký tự A", "ký tự a"),
    ("chữ B", "chữ bê"),
    ("ký tự 'C'", "ký tự xê"),
    ("chữ cái Z", "chữ cái dét"),
    ("kí tự w", "kí tự vê kép"),
    ("Anh M. đi bộ", "anh mờ đi bộ"),
    ("Vitamin G", "vitamin gờ"),
    ("L là tên riêng", "lờ là tên riêng"),
    ("#1", "thăng một"),
    ("#1kg", "thăng một ki lô gam"),

    # ══ 15. VIẾT TẮT TIẾNG VIỆT & HỌC VỊ ═════════════════════════════════════
    ("UBND", "uỷ ban nhân dân"),
    ("TP.HCM", "thành phố hồ chí minh"),
    ("TPHCM", "thành phố hồ chí minh"),
    ("CSGT", "cảnh sát giao thông"),
    ("LHQ", "liên hợp quốc"),
    ("CLB", "câu lạc bộ"),
    ("HLV", "huấn luyện viên"),
    ("TS", "tiến sĩ"),
    ("GS", "giáo sư"),
    ("THPT", "trung học phổ thông"),
    ("THCS", "trung học cơ sở"),
    ("theo QĐND đưa tin", "theo quân đội nhân dân đưa tin"),
    ("họp BCH", "họp ban chấp hành"),
    ("vào BV khám", "vào bệnh viện khám"),
    ("Du lịch tại UAE.", "du lịch tại u a e."),
    ("v.v", "vân vân"),
    ("v/v", "về việc"),
    ("đ/c", "địa chỉ"),
    # Học vị có dấu chấm trước tên -> bỏ chấm.
    ("Th.S Nguyễn", "thạc sĩ nguyễn"),
    ("TS. Nguyễn", "tiến sĩ nguyễn"),
    ("GS. Trần", "giáo sư trần"),
    ("KS Trần", "kỹ sư trần"),
    ("BS. Lê", "bác sĩ lê"),
    ("Ông ấy là PGS.TS ngành AI.", "ông ấy là phó giáo sư tiến sĩ ngành <en>a i</en>."),

    # ══ 16. ACRONYM / ALPHANUMERIC / CAMELCASE (tiếng Anh) ════════════════════
    ("Mô hình B2B rất phổ biến.", "mô hình <en>b two b</en> rất phổ biến."),
    ("Tôi dùng camera K3.", "tôi dùng camera ca ba."),
    ("camera K8", "camera ca tám"),
    ("i9-14900K", "i chín mười bốn nghìn chín trăm ca"),  # hậu tố model, không phải tiền lóng
    ("Mã số A1B.", "mã số a một bê."),
    ("Tôi đang học về AI.", "tôi đang học về <en>a i</en>."),
    ("Dự án VYE.", "dự án <en>v y e</en>."),
    ("Chào mừng bạn đến với CTY.", "chào mừng bạn đến với <en>c t y</en>."),
    ("TÔI ĐI HỌC", "tôi đi học"),
    ("Dữ liệu dạng JSON.", "dữ liệu dạng <en>j son</en>."),
    ("Chỉ số VN-Index giảm.", "chỉ số <en>v n</en> index giảm."),
    ("Hệ điều hành MS DOS.", "hệ điều hành <en>m s dos</en>."),
    ("Dùng MI5 và MI6.", "dùng <en>m i five</en> và <en>m i six</en>."),
    ("Bảo mật 2FA.", "bảo mật <en>two f a</en>."),
    ("Máy tính TX-0.", "máy tính <en>t x zero</en>."),
    ("Thiết bị mã TX-0 vẫn còn trong bảo tàng.", "thiết bị mã <en>t x zero</en> vẫn còn trong bảo tàng."),
    ("Phong cách thời trang Y2K đang quay trở lại mạnh mẽ.", "phong cách thời trang y hai ca đang quay trở lại mạnh mẽ."),
    ("tôi đang ở Washington D.C", "tôi đang ở <en>washington d c</en>"),
    ("tôi đang ở Washington DC", "tôi đang ở <en>washington d c</en>"),
    ("chuẩn ISO", "chuẩn <en>iso</en>"),
    ("máy ảnh ISO 400", "máy ảnh <en>iso</en> bốn trăm"),
    # camelCase splitting.
    ("MixedCase Acronyms như ChatGPT hay Claude.", "mixed case acronyms như chat <en>g p t</en> hay claude."),
    ("getUserIDFromDB", "get user <en>i d</en> from <en>d b</en>"),
    ("getUserById", "get user by id"),
    ("setVolumeLevel", "set volume level"),
    ("iPhone", "i Phone"),
    ("ChatGPT", "chat <en>g p t</en>"),
    # Cụm tiếng Anh nối gạch ngang -> tách bằng khoảng trắng, không đọc "gạch ngang".
    ("text-to-speech", "text to speech"),
    ("end-to-end", "end to end"),
    ("state-of-the-art", "state of the art"),
    ("plug-and-play", "plug and play"),

    # ══ 17. ACRONYM NỐI "&" ══════════════════════════════════════════════════
    ("R&D", "<en>r and d</en>"),
    ("R & D", "<en>r and d</en>"),
    ("phòng R&D", "phòng <en>r and d</en>"),
    ("AT&T", "<en>a t and t</en>"),
    ("S&P 500", "<en>s and p</en> năm trăm"),
    ("M&A", "<en>m and a</en>"),
    ("A & B", "a và bê"),    # chữ cái thường -> "và"
    ("3 & 4", "ba và bốn"),
    ("A + B", "a cộng bê"),
    ("A = B", "a bằng bê"),

    # ══ 18. NHÃN SIZE QUẦN ÁO ═════════════════════════════════════════════════
    ("size M/L/XL", "size <en>m</en> <en>l</en> <en>x l</en>"),
    ("cỡ M", "cỡ <en>m</en>"),
    ("size S/M/L", "size <en>s</en> <en>m</en> <en>l</en>"),
    ("cỡ lớn", "cỡ lớn"),   # không phải nhãn size

    # ══ 19. CHỮ HOA & TỪ TIẾNG VIỆT VIẾT HOA ═════════════════════════════════
    ("CHƯƠNG 4", "chương bốn"),
    ("CHƯƠNG 4: MỞ ĐẦU", "chương bốn, mở đầu"),
    ("BÁO CÁO QUÝ 4", "báo cáo quý bốn"),
    ("MUA 2 SẢN PHẨM", "mua hai sản phẩm"),
    ("PHẦN 2 KẾT THÚC", "phần hai kết thúc"),
    ("đọc CHƯƠNG này", "đọc chương này"),
    ("đi trên ĐƯỜNG lớn", "đi trên đường lớn"),
    ("về PHƯỜNG nộp", "về phường nộp"),
    ("ăn BƯỞI ngọt", "ăn bưởi ngọt"),
    # Acronym có dấu nhưng KHÔNG phải âm tiết -> vẫn tách.
    ("giải ĐKVĐ này", "giải đê ca vê đê này"),
    ("ông ĐBQH phát biểu", "ông đại biểu quốc hội phát biểu"),

    # ══ 20. CÔNG THỨC HÓA HỌC ════════════════════════════════════════════════
    ("CO2", "xê ô hai"),
    ("CO2 và H2O", "xê ô hai và hát hai ô"),
    ("HClO", "hát xê lờ ô"),
    ("dung dịch HClO mạnh", "dung dịch hát xê lờ ô mạnh"),
    ("Cấu trúc Benzen C6H6 có vòng thơm đặc trưng.", "cấu trúc benzen xê sáu hát sáu có vòng thơm đặc trưng."),
    ("Hóa chất: NaCl, NaOH, HCl, HClO, NaClO, ZnO, CuO, FeO, HCN, HF, NaCN, NaBr, KI, KOH, KCl, KBr, MgO", "hóa chất, nờ a xê lờ, nờ a ô hát, hát xê lờ, hát xê lờ ô, nờ a xê lờ ô, dét nờ ô, xê u ô, ép e ô, hát xê nờ, hát ép, nờ a xê nờ, nờ a bê rờ, ca i, ca ô hát, ca xê lờ, ca bê rờ, mờ gờ ô"),
    ("CH3COOH + NaOH → CH3COONa", "xê hát ba xê ô ô hát cộng nờ a ô hát đến xê hát ba xê ô ô nờ a"),
    ("Phản ứng: 2H2 + O2 → 2H2O", "phản ứng, hai hát hai cộng ô hai đến hai hát hai ô"),

    # ══ 21. CÔNG THỨC TOÁN ════════════════════════════════════════════════════
    # Ký hiệu (đọc trực tiếp, không cần tag).
    ("Nếu x > 5 và y ≤ 10 thì xấp xỉ ≈ 0.", "nếu ích lớn hơn năm và y nhỏ hơn hoặc bằng mười thì xấp xỉ xấp xỉ không."),
    ("Biểu thức ≥ 10.", "biểu thức lớn hơn hoặc bằng mười."),
    ("x ∈ ℝ", "ích thuộc tập số thực"),
    ("x ∈ ℤ", "ích thuộc tập số nguyên"),
    ("∫f(x)dx", "tích phân ép, ích, dx"),
    ("∂f/∂x", "đạo hàm riêng ép trên đạo hàm riêng ích"),
    ("y''", "y phẩy phẩy"),
    ("|x| < 5", "giá trị tuyệt đối của ích nhỏ hơn năm"),
    ("x⁴", "ích mũ bốn"),
    ("aᵢ", "a i"),
    ("Giới hạn lim(x→0) sin(x)/x = 1.", "giới hạn lim, ích đến không, sin, ích, trên ích bằng một."),
    # Dấu trừ trong công thức (số mũ / hệ số dính) -> "trừ"; văn xuôi giữ phẩy.
    # "4ac" trong cụm công thức được tách biến -> "bốn a xê".
    ("b² - 4ac", "bê bình phương trừ bốn a xê"),
    # ══ CÔNG THỨC TRẦN (không bọc <math>): cụm token dạng toán có dấu mạnh
    # (= √ ∫ ± mũ/chỉ số) được tách biến + giai thừa + trừ nhị phân ═══════════
    ("phương trình ax² + bx + c = 0", "phương trình a ích bình phương cộng bê ích cộng xê bằng không"),
    ("biệt thức Δ = b² - 4ac", "biệt thức đen ta bằng bê bình phương trừ bốn a xê"),
    ("công thức cos2x = 1 - 2sin²x", "công thức cos hai ích bằng một trừ hai sin bình phương ích"),
    ("E = mc² nổi tiếng", "e bằng mờ xê bình phương nổi tiếng"),
    ("trọng lượng P = mg", "trọng lượng phê bằng mờ gờ"),
    ("động năng bằng ½mv²", "động năng bằng một phần hai mờ vê bình phương"),
    ("khoảng 6,022 x 10²³ hạt", "khoảng sáu phẩy không hai hai nhân mười mũ hai mươi ba hạt"),
    # ══ Ký tự từng bị XÓA IM LẶNG — phát hiện bởi test bất biến (audit) ══════
    # Ký hiệu đồng ₫ (U+20AB): thiếu hẳn trong bảng tiền tệ.
    ("giá niêm yết 250.000₫ một suất", "giá niêm yết hai trăm năm mươi nghìn đồng một suất"),
    ("tổng cộng ₫1.200.000 phải trả", "tổng cộng một triệu hai trăm nghìn đồng phải trả"),
    # Ký hiệu độ gộp một ký tự ℃ (U+2103) / ℉ (U+2109).
    ("nhiệt độ ngoài trời 38℃", "nhiệt độ ngoài trời ba mươi tám độ xê"),
    ("nước sôi ở 212℉", "nước sôi ở hai trăm mười hai độ ép"),
    # Gạch nối kiểu chữ ‐ (U+2010) / ‑ (U+2011): trước bị xóa, các từ dính liền.
    ("công nghệ text‐to‐speech", "công nghệ text to speech"),
    # Mũi tên logic: chỉ "→" có trong bảng, các dạng còn lại bị xóa.
    ("nếu a lớn hơn b ⇒ a bình phương lớn hơn", "nếu a lớn hơn bê suy ra a bình phương lớn hơn"),
    ("quan hệ hai chiều a ⇔ b", "quan hệ hai chiều a tương đương bê"),
    ("tồn tại ∃ x thuộc tập số thực", "tồn tại tồn tại ích thuộc tập số thực"),
    # Số mũ ÂM viết bằng ⁻ (U+207B): trước đây dấu bị nuốt -> "mười lập phương".
    ("nồng độ 10⁻³ mol trên lít", "nồng độ mười mũ trừ ba mol trên lít"),
    ("sai số cỡ 10⁻⁶", "sai số cỡ mười mũ trừ sáu"),
    ("hằng số 6,626 x 10⁻³⁴ jun giây", "hằng số sáu phẩy sáu hai sáu nhân mười mũ trừ ba mươi bốn jun giây"),
    ("biểu thức 2⁻¹", "biểu thức hai mũ trừ một"),
    ("sản lượng tăng 10⁺⁶ lần", "sản lượng tăng mười mũ sáu lần"),
    ("hàm f(x) = eˣ rất đẹp", "hàm ép, ích, bằng e mũ ích rất đẹp"),
    ("Ký hiệu Σ là tổng", "ký hiệu xích ma là tổng"),
    ("câu trả lời là 5! = 120 cách", "câu trả lời là năm giai thừa bằng một trăm hai mươi cách"),
    ("độ phức tạp O(n!) bùng nổ", "độ phức tạp ô, nờ giai thừa, bùng nổ"),
    ("công thức tổ hợp C(n,k) = n!/(k!(n-k)!)",
     "công thức tổ hợp xê, nờ, ca, bằng nờ giai thừa trên, ca giai thừa nờ trừ ca, giai thừa"),
    ("số phức z = 5 - 2i", "số phức dét bằng năm trừ hai i"),
    ("giải log₃(x - 1) = 2", "giải log ba, ích trừ một, bằng hai"),
    ("đạo hàm là -sin x, đổi dấu", "đạo hàm là âm sin ích, đổi dấu"),
    # Từ Việt không dấu cạnh công thức KHÔNG bị hút vào cụm ("khi" giữ nguyên).
    ("Tổng Σ(1/2ⁿ) khi n tiến ra vô cùng", "tổng xích ma, một trên hai mũ nờ, khi nờ tiến ra vô cùng"),
    # ...kể cả khi có dấu phẩy đuôi ("thi,") hoặc đứng cạnh token chứa √ ("ta").
    ("trước khi thi, cos 60° = 1/2 nhé", "trước khi thi, cos sáu mươi độ bằng một trên hai nhé"),
    ("mẫu của 1/√3, ta nhân cả tử và mẫu với √3",
     "mẫu của một trên căn bậc hai ba, ta nhân cả tử và mẫu với căn bậc hai ba"),
    # "2bc" tách biến xong không bị pass đơn vị đọc "2 b" thành tỷ.
    ("a² = b² + c² - 2bc cos A", "a bình phương bằng bê bình phương cộng xê bình phương trừ hai bê xê cos a"),
    # Vi phân dx/du/dv luôn thuộc cụm công thức.
    ("tính ∫sin x dx trên đoạn", "tính tích phân sin ích đê ích trên đoạn"),
    # Trừ giữa hai biến chữ thường đơn lẻ.
    ("phép chia đa thức cho x - a", "phép chia đa thức cho ích trừ a"),
    # Chữ HOA đơn + chấm ở CUỐI câu giữ dấu chấm (không phải viết tắt tên).
    ("hệ thức U = IR.", "hệ thức u bằng i rờ."),
    # Hệ số trước công thức hóa học tách rời; đơn vị mũ Unicode về vuông/khối.
    ("phản ứng 6CO2 + 6H2O cần ánh sáng", "phản ứng sáu xê ô hai cộng sáu hát hai ô cần ánh sáng"),
    ("phản ứng 2HCl sủi bọt", "phản ứng hai hát xê lờ sủi bọt"),
    ("rộng 68 m² và chứa 1.200 m³", "rộng sáu mươi tám mét vuông và chứa một nghìn hai trăm mét khối"),
    ("2x - 3", "hai ích trừ ba"),
    ("8x - mà với lại", "tám ích, mà với lại"),
    # Tag <math>: tách cụm biến thành chữ rời, giữ tên hàm.
    ("<math>b² - 4ac</math>", "bê bình phương trừ bốn a xê"),
    ("<math>∫f dx</math>", "tích phân ép đê ích"),
    ("<math>∮ E·dl = 0</math>", "tích phân đường e nhân đê lờ bằng không"),
    ("<math>E = mc²</math>", "e bằng mờ xê bình phương"),
    ("<math>ax² + bx + c = 0</math>", "a ích bình phương cộng bê ích cộng xê bằng không"),
    ("<math>dy/dx</math>", "đê y trên đê ích"),
    ("<math>sin(x) + cos(x)</math>", "sin, ích, cộng cos, ích"),
    # Dấu trừ đơn nguyên trước biến (-b) -> "âm".
    ("<math>y = -x</math>", "y bằng âm ích"),
    ("<math>-a + b</math>", "âm a cộng bê"),
    ("<math>x = (-b ± √(b² - 4ac)) / 2a</math>",
     "ích bằng, âm bê cộng trừ căn bậc hai bê bình phương trừ bốn a xê, trên hai a"),
    # Trong <math>: giai thừa + dấu trừ nhị phân.
    ("<math>n!</math>", "nờ giai thừa"),
    ("<math>5!</math>", "năm giai thừa"),
    ("<math>c-d</math>", "xê trừ đê"),
    ("<math>a-b-c</math>", "a trừ bê trừ xê"),
    ("<math>(a+b)/(c-d)</math>", "a cộng bê, trên, xê trừ đê"),
    ("<math>n! / (k!(n-k)!)</math>", "nờ giai thừa trên, ca giai thừa nờ trừ ca, giai thừa"),
    ("<math>5 - 3 = 2</math>", "năm trừ ba bằng hai"),
    ("ac quy", "ac quy"),   # ngoài tag KHÔNG bị tách
    ("√4", "căn bậc hai bốn"),
    ("∛8", "căn bậc ba tám"),
    ("⅙ ⅛ ⅜ ⅝ ⅞", "một phần sáu một phần tám ba phần tám năm phần tám bảy phần tám"),

    # ══ 22. URL / EMAIL / KỸ THUẬT ═══════════════════════════════════════════
    ("Truy cập https://vieneu.io để biết thêm chi tiết.", "truy cập hát tê tê phê ét hai chấm gạch chéo gạch chéo vieneu chấm i ô để biết thêm chi tiết."),
    ("Website www.google.com rất hữu ích.", "website vê kép vê kép vê kép chấm google chấm com rất hữu ích."),
    ("Trang chủ là https://openai.com.", "trang chủ là hát tê tê phê ét hai chấm gạch chéo gạch chéo openai chấm com."),
    ("Tài liệu nằm ở www.example.org/docs.", "tài liệu nằm ở vê kép vê kép vê kép chấm example chấm o rờ gờ gạch chéo docs."),
    ("Repo nằm ở github.com/user/project.", "repo nằm ở github chấm com gạch chéo user gạch chéo project."),
    ("Repo nằm ở https://github.com/user/project-v2.", "repo nằm ở hát tê tê phê ét hai chấm gạch chéo gạch chéo github chấm com gạch chéo user gạch chéo project gạch nối vê hai."),
    ("Tài liệu đọc tại docs.python.org.", "tài liệu đọc tại docs chấm python chấm o rờ gờ."),
    ("File tải tại ftp://example.org/data.zip.", "file tải tại ép tê phê hai chấm gạch chéo gạch chéo example chấm o rờ gờ gạch chéo data chấm zip."),
    ("Máy chủ dự phòng là http://127.0.0.1:5000/health.", "máy chủ dự phòng là hát tê tê phê hai chấm gạch chéo gạch chéo một hai bảy chấm không chấm không chấm một hai chấm năm không không không gạch chéo health."),
    ("API local chạy ở http://localhost:8080/api/v2?lang=vi#top.", "<en>a p i</en> local chạy ở hát tê tê phê hai chấm gạch chéo gạch chéo localhost hai chấm tám không tám không gạch chéo api gạch chéo vê hai hỏi chấm lang bằng vi thăng top."),
    # URL có path tiếng Việt (không lòi dấu "/").
    ("truy cập https://abc.com/báo-cáo.", "truy cập hát tê tê phê ét hai chấm gạch chéo gạch chéo abc chấm com gạch chéo báo gạch nối cáo."),
    ("abc.com/báo-cáo", "abc chấm com gạch chéo báo gạch nối cáo"),
    ("https://abc.com/tài-liệu/mới", "hát tê tê phê ét hai chấm gạch chéo gạch chéo abc chấm com gạch chéo tài gạch nối liệu gạch chéo mới"),
    # Email.
    ("Liên hệ qua email pnnbao@gmail.com nhé.", "liên hệ qua email phê nờ nờ bao a còng gmail chấm com nhé."),
    # Câu thuần Anh (không từ Việt) -> email đọc kiểu Anh ("at", "dot").
    ("Email: contact@example.com", "email, <en>contact</en> at <en>example</en> dot <en>com</en>"),
    ("Email công việc: admin@fpt.vn", "email công việc, admin a còng ép phê tê chấm vê nờ"),
    ("Liên hệ hotmail: test@hotmail.com", "liên hệ hotmail, test a còng hotmail chấm com"),
    ("Hãy gửi email đến support@example.com.", "hãy gửi email đến support a còng example chấm com."),
    ("Email với tên miền lạ: user@domain.tech", "email với tên miền lạ, user a còng domain chấm tech"),
    ("Liên hệ qua email research.ai+test@example-domain.org.", "liên hệ qua email research chấm ai cộng test a còng example gạch nối domain chấm o rờ gờ."),
    ("Gửi báo cáo đến admin_v2@server.ai.", "gửi báo cáo đến admin gạch dưới vê hai a còng server chấm ai."),
    # Chuỗi kỹ thuật (IP, version, đường dẫn, file).
    ("Địa chỉ IP là 192.168.1.1 hoặc 10.0.0.1.", "địa chỉ <en>i p</en> là một chín hai chấm một sáu tám chấm một chấm một hoặc một không chấm không chấm không chấm một."),
    ("Phiên bản phần mềm là 1.25.3.4", "phiên bản phần mềm là một chấm hai năm chấm ba chấm bốn"),
    ("IPv6 là 2001:0db8:85a3:0000:0000:8a2e:0370:7334", "<en>i p v</en> sáu là hai không không một hai chấm không đê bê tám hai chấm tám năm a ba hai chấm không không không không hai chấm không không không không hai chấm tám a hai e hai chấm không ba bảy không hai chấm bảy ba ba bốn"),
    ("Mã này là 192.16.2", "mã này là một chín hai chấm một sáu chấm hai"),
    ("Đường dẫn Windows: C:\\Users\\dev\\data\\report_2026-03-11.log.", "đường dẫn windows, xê hai chấm gạch chéo users gạch chéo dev gạch chéo data gạch chéo report gạch dưới hai không hai sáu gạch nối không ba gạch nối một một chấm log."),
    ("Username của tôi là user_2024_dev.", "username của tôi là user gạch dưới hai không hai bốn gạch dưới dev."),
    ("File backup nằm ở /home/user/data_v3.2.tar.gz.", "file backup nằm ở gạch chéo home gạch chéo user gạch chéo data gạch dưới vê ba chấm hai chấm tar chấm gờ dét."),
    ("Log lỗi ghi tại error_log_2024-10-21.txt.", "log lỗi ghi tại error gạch dưới log gạch dưới hai không hai bốn gạch nối một không gạch nối hai một chấm tê ích tê."),
    # Path/URL trong câu tiếng Việt: separator đọc kiểu Việt (gạch chéo/gạch nối),
    # từ có trong dict để trần cho G2P, từ không dấu ngoài dict tách âm tiết Việt,
    # cụm toàn phụ âm đánh vần tên chữ Việt, đuôi file đánh vần ("phê y").
    ("Ảnh chụp bảng tin lớp cô đăng ở \\\\truong-mn\\thongbao\\thuc_don_tuan_32.jpg, mẹ nào cần thì tải.", "ảnh chụp bảng tin lớp cô đăng ở gạch chéo gạch chéo truong gạch nối mờ nờ gạch chéo thong bao gạch chéo thuc gạch dưới don gạch dưới tuan gạch dưới ba hai chấm giây phê gờ, mẹ nào cần thì tải."),
    ("Phòng vé lưu vé của đoàn vào \\\\phongve\\doan_cong_tac\\ve_may_bay_ha_noi.pdf nhé.", "phòng vé lưu vé của đoàn vào gạch chéo gạch chéo phong ve gạch chéo doan gạch dưới cong gạch dưới tac gạch chéo ve gạch dưới may gạch dưới bay gạch dưới ha gạch dưới noi chấm phê đê ép nhé."),
    # Path chỉ 1 backslash đầu (hay gặp khi copy văn bản làm mất 1 dấu \).
    ("Tài liệu lưu trong \\phongve\\tai_lieu\\huong_dan.pdf nhé.", "tài liệu lưu trong gạch chéo phong ve gạch chéo tai gạch dưới lieu gạch chéo huong gạch dưới dan chấm phê đê ép nhé."),
    # Query string sau TLD (?key=value) phải nằm trong URL, không đứt rời.
    ("Học viên tra cứu chứng chỉ tại tracuu.trungtamtinhoc.edu.vn?so=CC1204 nhé.", "học viên tra cứu chứng chỉ tại tra cuu chấm trung tam tin hoc chấm ê đu chấm vê nờ hỏi chấm so bằng xê xê một hai không bốn nhé."),
    # TLD mới (.dev) + đuôi io/edu đọc kiểu Việt, vn đọc "vi en".
    ("Anh tải bản desktop ở download.toolbox.dev/desktop/v1-8-3?os=windows giúp em.", "anh tải bản desktop ở download chấm toolbox chấm dev gạch chéo desktop gạch chéo vê một gạch nối tám gạch nối ba hỏi chấm os bằng windows giúp em."),
    ("Truy cập dataset.nlplab.io để tải dữ liệu nhé.", "truy cập dataset chấm nlplab chấm i ô để tải dữ liệu nhé."),
    # Tách hỗn hợp 3 hạng mảnh: âm tiết Việt / từ Anh top / cụm phụ âm đánh vần.
    # Từ Anh trong dict giữ khối ("smarthome"); mảnh lạ có nguyên âm ngoài top
    # wordlist không được cắt ("buildserver" nguyên khối, không "bui ldserver").
    ("Hướng dẫn ở https://hotro.smarthome24.vn/huong-dan nhé anh.", "hướng dẫn ở hát tê tê phê ét hai chấm gạch chéo gạch chéo ho tro chấm smarthome hai bốn chấm vê nờ gạch chéo huong gạch nối dan nhé anh."),
    ("Quy trình ở hr.tapdoanxyz.com nhé.", "quy trình ở hát rờ chấm tap doan ích y dét chấm com nhé."),
    ("Dịch vụ ở blogcongnghe.io nhé.", "dịch vụ ở blog cong nghe chấm i ô nhé."),
    ("Bản build ở buildserver.dev nhé.", "bản build ở buildserver chấm dev nhé."),
    # Câu thuần Anh KHÔNG Việt hóa viết tắt (VN giữ nguyên acronym chữ Anh).
    ("The VN team beat Thailand in the final match.", "the <en>v n</en> team beat thailand in the final match."),
    ("Our new office is located in TP.HCM near the river.", "our new office is located in <en>t p</en> dot <en>h c m</en> near the river."),
    ("Đội tuyển VN thắng trận chung kết.", "đội tuyển việt nam thắng trận chung kết."),
    # Viết tắt hành chính/đời sống mở rộng.
    ("Tra cứu điểm GPLX trên cổng dịch vụ công.", "tra cứu điểm giấy phép lái xe trên cổng dịch vụ công."),
    ("Bộ TN-MT vừa ban hành thông tư mới.", "bộ tài nguyên môi trường vừa ban hành thông tư mới."),
    ("Mang theo CMND hoặc CCCD.", "mang theo chứng minh nhân dân hoặc căn cước công dân."),
    ("Đội PCCC và CSGT phối hợp.", "đội phòng cháy chữa cháy và cảnh sát giao thông phối hợp."),
    ("BHYT chi trả 80%.", "bảo hiểm y tế chi trả tám mươi phần trăm."),
    ("Bộ GD&ĐT khẳng định giữ nguyên cấu trúc đề thi.", "bộ giáo dục đào tạo khẳng định giữ nguyên cấu trúc đề thi."),
    ("Cán bộ phòng LĐ-TB&XH xuống xã trao quà.", "cán bộ phòng lao động thương binh xã hội xuống xã trao quà."),
    ("Đường sách nằm ngay Q.1, đi bộ năm phút.", "đường sách nằm ngay quận một, đi bộ năm phút."),
    ("Trụ sở chuyển về P.Bến Nghé rồi.", "trụ sở chuyển về phường bến nghé rồi."),
    ("Cty TNHH MTV của anh ấy có bốn nhân viên.", "công ty trách nhiệm hữu hạn một thành viên của anh ấy có bốn nhân viên."),
    ("TGĐ mới đổi quy trình, bà PTGĐ xuống xưởng.", "tổng giám đốc mới đổi quy trình, bà phó tổng giám đốc xuống xưởng."),
    ("Đọc BCTC trước kỳ ĐHĐCĐ nhé.", "đọc báo cáo tài chính trước kỳ đại hội đồng cổ đông nhé."),
    ("Thi TOEIC và dự SEA Games, xem ASIAD.", "thi <en>toeic</en> và dự <en>sea games</en>, xem a si át."),
    # Acronym quen đọc như TỪ (WORD_LIKE_ACRONYMS).
    ("Trọng tài xem VAR trận EURO do UEFA tổ chức.", "trọng tài xem <en>var</en> trận <en>euro</en> do <en>uefa</en> tổ chức."),
    ("FED tăng lãi suất khiến NASDAQ đỏ lửa.", "<en>fed</en> tăng lãi suất khiến <en>nasdaq</en> đỏ lửa."),
    ("Gửi ảnh GIF qua WIFI nhé.", "gửi ảnh <en>gif</en> qua <en>wifi</en> nhé."),
    ("Lắp thẻ SIM 1 vào khay, bật đèn LED lên.", "lắp thẻ <en>sim</en> một vào khay, bật đèn <en>led</en> lên."),
    ("Phiên tòa lừa đảo XKLĐ hôm qua.", "phiên tòa lừa đảo xuất khẩu lao động hôm qua."),
    ("Khoa CNTT và sàn TMĐT đang hot.", "khoa công nghệ thông tin và sàn thương mại điện tử đang hot."),
    # T2..T7/CN là thứ CHỈ KHI có từ dẫn thời gian; "Model T2" giữ nguyên.
    ("Hẹn gặp sáng T2 tuần sau nhé.", "hẹn gặp sáng thứ hai tuần sau nhé."),
    ("Lịch học từ T2 đến T6, nghỉ T7 và CN.", "lịch học từ thứ hai đến thứ sáu, nghỉ thứ bảy và chủ nhật."),
    ("Model T2 của hãng ra mắt.", "model tê hai của hãng ra mắt."),
    # Exception camelCase mask sớm: "arXiv" không bị xé "ar Xiv" (xiv = số La Mã).
    ("Bài báo mới đăng trên arXiv hôm qua.", "bài báo mới đăng trên <en>arxiv</en> hôm qua."),
    # Từ ghép toàn tiếng Anh ("ielts"+"zone") giữ khối, G2P tự cắt ở tầng phoneme.
    ("Trả kết quả qua ketqua.ieltszone.edu.vn?sbd=IZ0457 nhé.", "trả kết quả qua ket qua chấm ieltszone chấm ê đu chấm vê nờ hỏi chấm ét bê đê bằng i dét không bốn năm bảy nhé."),
    ("Gửi tới pnnbao@gmail.com nhé.", "gửi tới phê nờ nờ bao a còng gmail chấm com nhé."),
    # Vần "uu" ("lưu trữ" không dấu) và camelCase thắng entry rác trong dict.
    ("Ảnh bìa lưu ở \\\\toasoan\\luutru\\bia_tap_chi.pdf nhé.", "ảnh bìa lưu ở gạch chéo gạch chéo toa soan gạch chéo luu tru gạch chéo bia gạch dưới tap gạch dưới chi chấm phê đê ép nhé."),
    ("Bản khai thuế lưu ở C:\\Thue\\CaNhan\\to_khai_tncn.pdf nhé.", "bản khai thuế lưu ở xê hai chấm gạch chéo thue gạch chéo ca nhan gạch chéo to gạch dưới khai gạch dưới tê nờ xê nờ chấm phê đê ép nhé."),
    ("Nhớ sao lưu sổ tay của mẹ trong C:\\CongThucNauAn\\so_tay_mon_bac.docx đấy con.", "nhớ sao lưu sổ tay của mẹ trong xê hai chấm gạch chéo cong thuc nau an gạch chéo so gạch dưới tay gạch dưới mon gạch dưới bac chấm docx đấy con."),
    ("Kết xuất căn hộ mẫu nằm ở D:\\KienTruc\\CanHoMau\\phoi_canh_phong_khach.png nhé.", "kết xuất căn hộ mẫu nằm ở đê hai chấm gạch chéo kien truc gạch chéo can ho mau gạch chéo phoi gạch dưới canh gạch dưới phong gạch dưới khach chấm phê nờ gờ nhé."),
    ("Cháu viết script đổi tên ảnh ở /home/scripts/doi_ten_anh.py, chạy một lệnh là xong.", "cháu viết script đổi tên ảnh ở gạch chéo home gạch chéo scripts gạch chéo doi gạch dưới ten gạch dưới anh chấm phê y, chạy một lệnh là xong."),
    ("Bản demo giọng đọc xuất ra D:\\TTS\\Demo\\giong_nu_mien_bac_v2.wav rồi anh.", "bản demo giọng đọc xuất ra đê hai chấm gạch chéo tê tê ét gạch chéo demo gạch chéo giong gạch dưới nu gạch dưới mien gạch dưới bac gạch dưới vê hai chấm wav rồi anh."),
    # Số 2/4 kẹp giữa chữ thường là viết tắt to/for tiếng Anh -> "two"/"four".
    ("mô hình text2text đang hot", "mô hình text two text đang hot"),
    ("kinh doanh b2b khó lắm", "kinh doanh <en>b</en> two <en>b</en> khó lắm"),
    ("dịch vụ food2door giao tận nơi", "dịch vụ food two door giao tận nơi"),
    # Câu thuần Anh (>=3 từ tiếng Anh, không từ Việt) -> số/ký hiệu đọc kiểu Anh.
    ("I have 3 dogs and 2 cats.", "i have three dogs and two cats."),
    ("The meeting starts at 10:30 tomorrow.", "the meeting starts at ten thirty tomorrow."),
    ("We got a 50% discount on Windows 11.", "we got a fifty percent discount on windows eleven."),
    ("The file is 2.5 MB.", "the file is two point five megabytes."),
    ("Download it from github.com/user/project now.", "download it from <en>github</en> dot <en>com</en> slash <en>user</en> slash <en>project</en> now."),
    # Mẩu trơ không đủ từ tiếng Anh -> vẫn đọc kiểu Việt.
    ("Arsenal 3-0 Chelsea", "arsenal ba không chelsea"),
    ("3.46 USD/gallon", "ba chấm bốn sáu <en>u s d</en> trên <en>gallon</en>"),
    ("Chuỗi có placeholder ___PROTECTED_EN_TAG_0___ để kiểm tra xung đột.", "chuỗi có placeholder protected en tag không để kiểm tra xung đột."),
    ("Câu lệnh SQL: SELECT * FROM users WHERE id=1;", "câu lệnh <en>s q l</en>, <en>select</en> sao <en>from</en> users <en>where</en> id bằng một"),
    ("Cú pháp: [x**2 for x in range(10) if x%2 == 0] trong Python.", "cú pháp, ích sao sao hai for ích in range mười, if ích phần trăm hai bằng bằng không trong python."),
    ("WebAssembly (Wasm) cho phép chạy code C++ trên trình duyệt.", "web assembly, wasm, cho phép chạy code xê cộng cộng trên trình duyệt."),
    ("Triển khai Kubernetes (K8s) trên cụm server bare-metal.", "triển khai kubernetes, ca tám ét, trên cụm server bare metal."),
    ("GPU NVIDIA RTX 4090 có 24GB GDDR6X VRAM.", "<en>g p u</en> <en>n v d a</en> <en>r t x</en> bốn nghìn không trăm chín mươi có hai mươi bốn <en>gigabyte</en> gờ đê đê rờ sáu ích <en>v ram</en>."),
    ("Định luật bảo toàn năng lượng: E_in = E_out + ΔE_system.", "định luật bảo toàn năng lượng, e in bằng e out cộng đen ta e system."),

    # ══ 23. DẤU NHÁY / NGOẶC / DẤU CÂU ════════════════════════════════════════
    ("(text in brackets)", "text in brackets"),
    ("[text in brackets]", "text in brackets"),
    ("(giờ Mỹ)", "giờ mỹ"),
    ("hiệu lực từ 0h01 (giờ Mỹ), trong vòng", "hiệu lực từ không giờ một phút, giờ mỹ, trong vòng"),
    ("kết thúc (0h01).", "kết thúc, không giờ một phút."),
    ("“Lời chào cao hơn mâm cỗ”", "lời chào cao hơn mâm cỗ"),
    ("‘Trân trọng’", "trân trọng"),
    ("Giá của 'Sản phẩm' này là $10", "giá của sản phẩm này là mười <en>u s d</en>"),
    ("Cậu ấy đúng là một 'workaholic', làm việc 12 tiếng mỗi ngày.", "cậu ấy đúng là một workaholic, làm việc mười hai tiếng mỗi ngày."),
    ("A' là một ký tự đặc biệt", "a phẩy là một ký tự đặc biệt"),
    ("Đây là phút 1'", "đây là phút một phẩy"),
    ("I don't know why", "i don't know why"),
    ("It's a beautiful day", "it's a beautiful day"),
    # Contraction chữ cái đơn (I'm, I'll...) phải giữ nguyên vẹn, không tách "i 'm".
    ("I'm đi chợ", "i'm đi chợ"),
    # Nháy cong U+2019 (bàn phím/Word) quy về nháy thẳng để contraction khớp dict.
    ("I’m đi chợ", "i'm đi chợ"),
    ("she’s xinh thật", "she's xinh thật"),
    ("don’t lo lắng", "don't lo lắng"),
    ("I'll gọi lại cho bạn sau", "i'll gọi lại cho bạn sau"),
    ("we're going to school and tôi không biết liệu she's beautiful", "we're going to school and tôi không biết liệu she's beautiful"),
    ("Giá SP500 hôm nay là 4.200,5 điểm", "giá ét pê năm trăm hôm nay là bốn nghìn hai trăm phẩy năm điểm"),
    ("chỉ số là 7,05 - đường huyết là 1.8", "chỉ số là bảy phẩy không năm, đường huyết là một chấm tám"),
    ("ta có !hôm nay thật kì lạ; ta sẽ đi,chơi", "ta có! hôm nay thật kì lạ, ta sẽ đi, chơi"),
    ("Tọa độ (-2.5;0)", "tọa độ, âm hai chấm năm, không"),
    ("Tọa độ GPS: 10°46'37\"N 106°41'43\"E (TP. Hồ Chí Minh).", "tọa độ <en>g p s</en>, mười độ bốn mươi sáu phẩy ba mươi bảy phẩy phẩy nờ một trăm lẻ sáu độ bốn mươi mốt phẩy bốn mươi ba phẩy phẩy e, thành phố. hồ chí minh."),
    ("EPS quý này đạt $3.45.Tiếng Việt có dấu: Hoà, Hòa, Hòa.", "<en>e p s</en> quý này đạt ba chấm bốn lăm <en>u s d</en>. tiếng việt có dấu, hoà, hòa, hòa."),
    ("tôi đang đi du lịch Đà Lạt với người yêu cũ...", "tôi đang đi du lịch Đà Lạt với người yêu cũ."),
    # Gộp dấu câu lặp.
    ("Trời ơi!!!", "trời ơi!"),
    ("Thật sao???", "thật sao?"),
    ("Cái gì?!", "cái gì?"),
    ("Hả!?!?", "hả!"),

    # ══ 24. THẺ <en> & CẤU TRÚC VĂN BẢN ══════════════════════════════════════
    ("<en>Hello</en>", "<en>Hello</en>"),
    ("<en>Hello 123</en>", "<en>Hello 123</en>"),
    ("Xin chào <en>Good morning</en>", "xin chào <en>Good morning</en>"),
    ("Ngày 21/02 <en>February 21</en>", "ngày hai mươi mốt tháng hai <en>February 21</en>"),
    ("<en>AI</en> là trí tuệ nhân tạo", "<en>AI</en> là trí tuệ nhân tạo"),
    ("Chào <en>world</en> xinh đẹp", "chào <en>world</en> xinh đẹp"),
    ("Đoạn 1.\nĐoạn 2.", "đoạn một.\nđoạn hai."),
    ("\n12 tiêm kích", "\nmười hai tiêm kích"),
    # Xuống dòng là ranh giới câu: đề mục HOA (không dấu chấm cuối) được xét riêng nên
    # từ Việt không dấu (LAO, KHEN) KHÔNG bị đọc thành <en>. Xem issue #177.
    ("VỀ HUÂN CHƯƠNG LAO ĐỘNG\nCông đoàn Dệt May.",
     "về huân chương lao động\ncông đoàn dệt may."),
    ("II. BẰNG KHEN\nĐồng chí Bế Thị Hòa.",
     "hai. bằng khen\nđồng chí bế thị hòa."),

    # ══ 24b. KÝ TỰ UNICODE ẨN (dán từ Word/web/PDF) — issue #177 ═══════════════
    # Khoảng trắng Unicode lạ -> dấu cách thường (nếu lọt qua sẽ thành token OOV
    # khiến TTS đọc ra 'tiếng lạ' giữa câu). Dùng escape backslash-u cho rõ ràng.
    ("Huân chương Lao động", "huân chương lao động"),   # NBSP
    ("Huân　chương", "huân chương"),                          # ideographic space
    ("Lao động bền", "lao động bền"),                   # thin + narrow NBSP
    ("1 000 000", "một triệu"),                         # narrow NBSP phân nhóm
    ("Trời mưa Gió to", "trời mưa gió to"),                   # line separator
    # Ký tự zero-width bị loại bỏ hẳn (không tạo khoảng trắng).
    ("thành​phố", "thành phố"),                              # ZWSP -> dấu cách
    ("﻿Việt‌Nam", "việt nam"),                         # BOM bỏ, ZWNJ -> dấu cách
    ("mềm­mại", "mềm mại"),                                  # soft hyphen -> dấu cách
    # Ký tự điều khiển bị loại bỏ.
    ("HàNội", "hà nội"),                                    # form feed (-> dấu cách)

    # ══ 25. CÂU THỰC TẾ (smoke test) ══════════════════════════════════════════
    ("Ngày 21/02/2025 lúc 14h30, giá vàng đạt 100$ tại TPHCM",
     "ngày hai mươi mốt tháng hai năm hai nghìn không trăm hai mươi lăm lúc mười bốn giờ ba mươi phút, giá vàng đạt một trăm <en>u s d</en> tại thành phố hồ chí minh"),
    ("Thế kỷ XXI chứng kiến sự phát triển của <en>AI</en> và vũ trụ học",
     "thế kỷ hai mươi mốt chứng kiến sự phát triển của <en>AI</en> và vũ trụ học"),
    ("Đề án 06 và Chỉ thị 04", "đề án không sáu và chỉ thị không bốn"),
    ("Ông Lưu Trung Thái, Chủ tịch HĐQT MB cho biết, vốn hóa của ngân hàng đã tăng gần 10 lần kể từ năm 2017, đạt khoảng 8,5 tỷ USD, tạo nền tảng cho mục tiêu 10 tỷ USD vào năm 2027.",
     "ông lưu trung thái, chủ tịch hội đồng quản trị <en>m b</en> cho biết, vốn hóa của ngân hàng đã tăng gần mười lần kể từ năm hai nghìn không trăm mười bảy, đạt khoảng tám phẩy năm tỷ <en>u s d</en>, tạo nền tảng cho mục tiêu mười tỷ <en>u s d</en> vào năm hai nghìn không trăm hai mươi bảy."),
    ("Nếu đã từng đọc cuốn sách trên của Simon, hoặc đã xem anh thuyết trình về khái niệm tại sao trên diễn đàn TED.com, thì có lẽ bạn không còn xa lạ với vòng tròn vàng.",
     "nếu đã từng đọc cuốn sách trên của simon, hoặc đã xem anh thuyết trình về khái niệm tại sao trên diễn đàn tê e đê chấm com, thì có lẽ bạn không còn xa lạ với vòng tròn vàng."),
    ("Dân số thế giới khoảng 7,888,000,000 người (~7.9B).",
     "dân số thế giới khoảng bảy tỷ tám trăm tám mươi tám triệu người, khoảng bảy chấm chín tỷ."),
    ("Latency trung bình chỉ ~42ms / request qua REST API.",
     "latency trung bình chỉ khoảng bốn mươi hai mi li giây trên request qua <en>rest</en> <en>a p i</en>."),
    ("Dataset gồm 3.2M samples (~1.8TB audio).",
     "dataset gồm ba chấm hai triệu samples, khoảng một chấm tám <en>terabyte</en> audio."),
    ("CPU Core i9-14900K chạy ở xung nhịp 6,0 GHz nhưng nhiệt độ lên tới 95°C.",
     "<en>c p u</en> core i chín mười bốn nghìn chín trăm ca chạy ở xung nhịp sáu gi ga héc nhưng nhiệt độ lên tới chín mươi lăm độ xê."),
    ("Thông tin này được Tập đoàn Hóa chất Đức Giang (DGC) công bố hôm 19/3 - hai ngày sau khi Bộ Công an thông báo tạm giam ông Đào Hữu Huyền",
     "thông tin này được tập đoàn hóa chất đức giang, <en>d g c</en>, công bố hôm mười chín tháng ba, hai ngày sau khi bộ công an thông báo tạm giam ông đào hữu huyền"),
    ("Vi khuẩn kháng thuốc Methicillin-resistant Staphylococcus aureus (MRSA).",
     "vi khuẩn kháng thuốc methicillin resistant staphylococcus aureus, <en>m r s a</en>."),
    ("Dom Studio cho đăng tải tập 17 Skippy Toilet Multiverse với rất nhiều tinh tiết đáng chú ý và cả đáng sợ nữa.",
     "dom studio cho đăng tải tập mười bảy skippy toilet multiverse với rất nhiều tinh tiết đáng chú ý và cả đáng sợ nữa."),
]


@pytest.mark.parametrize("input_text, expected", TEST_CASES)
def test_normalize(normalizer, input_text, expected):
    actual = normalizer.normalize(input_text)
    actual_clean = " ".join(actual.split()).lower()
    expected_clean = " ".join(expected.split()).lower()
    assert actual_clean == expected_clean
