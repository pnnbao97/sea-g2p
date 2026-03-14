from sea_g2p import Normalizer

def test_issues():
    n = Normalizer()

    cases = [
        ("Nhiệt độ là -5°C", "nhiệt độ là âm năm độ xê"),
        ("Tọa độ (-2.5;0)", "tọa độ âm hai chấm năm không"),
        ("Họp lúc 8g sáng", "họp lúc tám giờ sáng"),
        ("090-123-4567", "không chín không một hai ba bốn năm sáu bảy"),
        ("ISO 9001:2015", "i s o chín nghìn không trăm lẻ một hai chấm hai nghìn không trăm mười lăm")
    ]

    for inp, exp in cases:
        actual = n.normalize(inp)
        print(f"Input: {inp}")
        print(f"Actual: {actual}")
        print(f"Expect: {exp}")
        print("-" * 20)

if __name__ == "__main__":
    test_issues()
