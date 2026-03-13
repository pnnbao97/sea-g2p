import re
# Re-exporting functions for backward compatibility and as a central entry point
from .technical import normalize_technical, normalize_emails, normalize_slashes, RE_TECHNICAL, RE_EMAIL
from .units import (
    expand_measurement, expand_currency, expand_compound_units,
    expand_scientific_notation, fix_english_style_numbers, expand_power_of_ten,
    _expand_number_with_sep, _expand_scientific, _expand_mixed_sep, _expand_single_sep
)
from .others import (
    normalize_others, normalize_acronyms, expand_roman, expand_letter,
    expand_abbreviations, expand_standalone_letters, expand_alphanumeric,
    expand_symbols, expand_prime, expand_temperatures, expand_unit_powers,
    RE_ACRONYMS_EXCEPTIONS as _RE_ACRONYMS_EXCEPTIONS
)
from .vi_resources import _combined_exceptions

# Compatibility for RE_ACRONYMS_EXCEPTIONS_RE if needed
_ACRONYMS_EXCEPTIONS_RE = [(re.compile(rf"\b{re.escape(k)}\b"), v) for k, v in sorted(_combined_exceptions.items(), key=lambda x: len(x[0]), reverse=True)]
