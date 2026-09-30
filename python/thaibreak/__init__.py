"""
ThaiBreak — Fast, high-accuracy Thai word segmentation and typographic line breaker.
"""

from .core import init, words, lines, wrap, display_width

__all__ = ["init", "words", "lines", "wrap", "display_width"]
try:
    from importlib.metadata import version as _version

    __version__ = _version("thaibreak")
except Exception:  # running from a source checkout
    __version__ = "unknown"
