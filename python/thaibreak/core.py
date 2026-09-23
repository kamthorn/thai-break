import ctypes
import os
from typing import List, Optional

_lib = None

def _find_lib():
    global _lib
    if _lib is not None:
        return _lib
    
    # Candidate library paths
    candidates = [
        os.path.join(os.path.dirname(__file__), "libthaibreak.so"),
        os.path.join(os.path.dirname(__file__), "../../rust/target/release/libthaibreak.so"),
        os.path.join(os.path.dirname(__file__), "../rust/target/release/libthaibreak.so"),
        "libthaibreak.so",
    ]
    for c in candidates:
        if os.path.exists(c):
            try:
                _lib = ctypes.CDLL(c)
                break
            except Exception:
                pass
    if _lib is None:
        try:
            _lib = ctypes.CDLL("libthaibreak.so")
        except Exception:
            pass
            
    if _lib is not None:
        _lib.thaibreak_init.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
        _lib.thaibreak_init.restype = ctypes.c_int

        _lib.thaibreak_tokenize.argtypes = [ctypes.c_char_p, ctypes.POINTER(ctypes.c_size_t)]
        _lib.thaibreak_tokenize.restype = ctypes.POINTER(ctypes.c_char_p)

        _lib.thaibreak_free_tokens.argtypes = [ctypes.POINTER(ctypes.c_char_p), ctypes.c_size_t]
        _lib.thaibreak_free_tokens.restype = None

        _lib.thaibreak_lines.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_int]
        _lib.thaibreak_lines.restype = ctypes.c_void_p

        _lib.thaibreak_wrap.argtypes = [ctypes.c_char_p, ctypes.c_size_t, ctypes.c_int]
        _lib.thaibreak_wrap.restype = ctypes.c_void_p

        _lib.thaibreak_display_width.argtypes = [ctypes.c_char_p]
        _lib.thaibreak_display_width.restype = ctypes.c_size_t

        _lib.thaibreak_free_string.argtypes = [ctypes.c_void_p]
        _lib.thaibreak_free_string.restype = None
    return _lib

def init(dict_path: str, bigram_path: Optional[str] = None) -> bool:
    lib = _find_lib()
    if not lib:
        return False
    b_dict = dict_path.encode("utf-8")
    b_bigram = bigram_path.encode("utf-8") if bigram_path else None
    return lib.thaibreak_init(b_dict, b_bigram) == 0

def words(text: str) -> List[str]:
    lib = _find_lib()
    if not lib or not text:
        return []
    b_text = text.encode("utf-8")
    count = ctypes.c_size_t()
    ptr = lib.thaibreak_tokenize(b_text, ctypes.byref(count))
    if not ptr:
        return []
    try:
        tokens = [ptr[i].decode("utf-8") for i in range(count.value)]
    finally:
        lib.thaibreak_free_tokens(ptr, count)
    return tokens

def lines(text: str, marker: str = "\u200b", is_html: bool = False) -> str:
    lib = _find_lib()
    if not lib or not text:
        return text
    b_text = text.encode("utf-8")
    b_marker = marker.encode("utf-8") if marker else None
    raw_ptr = lib.thaibreak_lines(b_text, b_marker, 1 if is_html else 0)
    if not raw_ptr:
        return text
    try:
        result = ctypes.string_at(raw_ptr).decode("utf-8")
    finally:
        lib.thaibreak_free_string(raw_ptr)
    return result

def wrap(text: str, width: int, is_html: bool = False) -> str:
    lib = _find_lib()
    if not lib or not text:
        return text
    b_text = text.encode("utf-8")
    raw_ptr = lib.thaibreak_wrap(b_text, width, 1 if is_html else 0)
    if not raw_ptr:
        return text
    try:
        result = ctypes.string_at(raw_ptr).decode("utf-8")
    finally:
        lib.thaibreak_free_string(raw_ptr)
    return result

def display_width(text: str) -> int:
    lib = _find_lib()
    if not lib or not text:
        return 0
    return lib.thaibreak_display_width(text.encode("utf-8"))

