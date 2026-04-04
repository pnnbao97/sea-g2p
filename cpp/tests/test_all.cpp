#include "sea_g2p/g2p.hpp"
#include "sea_g2p/vi_normalizer.hpp"

#include <iostream>
#include <vector>
#include <string>
#include <cassert>
#include <stdexcept>

#ifdef _WIN32
#include <windows.h>
#endif

#include <algorithm>
#include <iterator>
#include <sstream>

std::string clean_str(const std::string& str) {
    std::string s = str;
    std::transform(s.begin(), s.end(), s.begin(), ::tolower);
    std::istringstream iss(s);
    std::vector<std::string> words;
    std::string word;
    while (iss >> word) words.push_back(word);
    
    std::string result;
    for (size_t i = 0; i < words.size(); ++i) {
        result += words[i];
        if (i < words.size() - 1) result += " ";
    }
    return result;
}

void test_normalization() {
    std::cout << "--- Testing Normalization (Robust) ---" << std::endl;
    try {
        sea_g2p::Normalizer normalizer("vi");
        
        struct TestCase {
            std::string input;
            std::string expected;
        };

        std::vector<TestCase> test_cases = {
            // 1. Numbers & Separators
            {"0", "không"},
            {"11", "mười một"},
            {"21", "hai mươi mốt"},
            {"1.001", "một nghìn không trăm lẻ một"},
            {"1,000,000", "một triệu"},
            {"3,14", "ba phẩy một bốn"},
            {"-123", "âm một trăm hai mươi ba"},  // Corrected: small numbers are verbalized
            {"1234567", "một hai ba bốn năm sáu bảy"}, // 7+ digits are digit-by-digit
            
            // 2. Dates & Time
            {"21/02/2025", "ngày hai mươi mốt tháng hai năm hai nghìn không trăm hai mươi lăm"},
            {"14h30", "mười bốn giờ ba mươi phút"},
            {"23:59", "hai mươi ba giờ năm mươi chín phút"},
            
            // 3. Units & Currency
            {"100$", "một trăm u s d"},
            {"500 VND", "năm trăm việt nam đồng"},
            {"1000đ", "một nghìn đồng"},
            {"75%", "bảy mươi lăm phần trăm"},
            {"50km", "năm mươi ki lô mét"},
            {"1,5 ha", "một phẩy năm héc ta"},
            
            // 4. Abbreviations & Roman
            {"UBND", "uỷ ban nhân dân"},
            {"TP.HCM", "thành phố hồ chí minh"},
            {"Thế kỷ XXI", "thế kỷ hai mươi mốt"},
            
            // 5. English & Technical
            {"support@example.com", "support a còng example chấm com"},
            {"https://vieneu.io", "https vieneu chấm i o"},
            {"AI là trí tuệ", "a i là trí tuệ"},
            {"getUserById", "get user by id"},
            {"iPhone", "i Phone"},
            
            // 6. Mixed
            {"Ngày 21/02 lúc 14h30, giá vàng đạt 100$", 
             "ngày hai mươi mốt tháng hai lúc mười bốn giờ ba mươi phút, giá vàng đạt một trăm u s d"}
        };

        int fails = 0;
        for (const auto& tc : test_cases) {
            std::string result = clean_str(normalizer.normalize(tc.input));
            // Remove <en> tags for easier comparison in this basic test
            std::string clean_result;
            size_t pos = 0;
            while (pos < result.length()) {
                if (result.substr(pos, 4) == "<en>") pos += 4;
                else if (result.substr(pos, 5) == "</en>") pos += 5;
                else {
                    clean_result += result[pos];
                    pos++;
                }
            }
            clean_result = clean_str(clean_result);
            std::string clean_expected = clean_str(tc.expected);

            if (clean_result != clean_expected) {
                std::cerr << "FAIL Normalize: [" << tc.input << "] -> [" << clean_result << "] != [" << clean_expected << "]" << std::endl;
                fails++;
            }
        }
        std::cout << "Normalization: " << (test_cases.size() - fails) << "/" << test_cases.size() << " passed." << std::endl;
        if (fails > 0) throw std::runtime_error("Normalization tests failed");

    } catch (const std::exception& e) {
        std::cerr << "CRASH in test_normalization: " << e.what() << std::endl;
        throw;
    }
}


void test_bilingual(const std::string& dict_path) {
    std::cout << "--- Testing Bilingual Propagation ---" << std::endl;
    try {
        sea_g2p::G2PEngine g2p(dict_path);
        
        struct PropagationCase {
            std::string natural;
            std::string tagged;
        };

        std::vector<PropagationCase> test_cases = {
            // Test 1: Propagation to the right anchor
            {"Tôi muốn go to the market", "Tôi muốn <en>go to the market</en>"},
            
            // Test 2: Punctuation as boundary (to in 'túi to' is VI, to in 'market' is EN)
            {"go to the market. Mua một cái túi to.", "<en>go to the market</en>. Mua một cái túi to."},
            
            // Test 3: Common word disambiguation (me, no, can)
            {"hello give it to me", "hello <en>give it to me</en>"},
            {"hello I say no", "hello <en>I say no</en>"},
            {"I can do it", "<en>I can do it</en>"}
        };

        int fails = 0;
        for (const auto& tc : test_cases) {
            std::string p_nat = clean_str(g2p.phonemize(tc.natural));
            std::string p_tag = clean_str(g2p.phonemize(tc.tagged));
            
            if (p_nat != p_tag) {
                std::cerr << "FAIL Bilingual Propagation: [" << tc.natural << "]" << std::endl;
                std::cerr << "  Nat: " << p_nat << std::endl;
                std::cerr << "  Tag: " << p_tag << std::endl;
                fails++;
            }
        }
        std::cout << "Bilingual: " << (test_cases.size() - fails) << "/" << test_cases.size() << " passed." << std::endl;
        if (fails > 0) throw std::runtime_error("Bilingual tests failed");

    } catch (const std::exception& e) {
        std::cerr << "CRASH in test_bilingual: " << e.what() << std::endl;
        throw;
    }
}

void test_g2p_base(const std::string& dict_path) {
    std::cout << "--- Testing G2P (Base) ---" << std::endl;
    try {
        sea_g2p::G2PEngine g2p(dict_path);
        
        struct TestCase {
            std::string input;
            std::string expected;
        };

        std::vector<TestCase> test_cases = {
            {"không", "xˌoŋ"},
            {"một", "mˈo6t̪"},
            {"mười một", "mˈyə2j mˈo6t̪"},
            {"ba phẩy một bốn", "bˈaː fˈəɪ4 mˈo6t̪ bˈoɜn"},
            {"thứ nhất", "tˈyɜ ɲˈəɜt̪"},
            {"thành phố hồ chí minh", "tˈe-2ɲ fˈoɜ hˈo2 tʃˈiɜ mˈiɲ"},
            {"xin chào", "sˈin tʃˈaː2w"}
        };

        int fails = 0;
        for (const auto& tc : test_cases) {
            std::string result = clean_str(g2p.phonemize(tc.input));
            std::string expected = clean_str(tc.expected);
            if (result != expected) {
                std::cerr << "FAIL G2P: [" << tc.input << "] -> [" << result << "] != [" << expected << "]" << std::endl;
                fails++;
            }
        }
        std::cout << "G2P: " << (test_cases.size() - fails) << "/" << test_cases.size() << " passed." << std::endl;
        if (fails > 0) throw std::runtime_error("G2P tests failed");

    } catch (const std::exception& e) {
        std::cerr << "CRASH in test_g2p: " << e.what() << std::endl;
        throw;
    }
}

int main(int argc, char* argv[]) {
#ifdef _WIN32
    SetConsoleOutputCP(CP_UTF8);
#endif
    if (argc < 2) {
        std::cerr << "Usage: " << argv[0] << " <dict_path>" << std::endl;
        return 1;
    }

    std::string dict_path = argv[1];

    try {
        test_normalization();
        test_g2p_base(dict_path);
        test_bilingual(dict_path);
        std::cout << "ALL TESTS PASSED SUCCESSFULLY!" << std::endl;
    } catch (const std::exception& e) {
        std::cerr << "Tests failed: " << e.what() << std::endl;
        return 1;
    }

    return 0;
}
