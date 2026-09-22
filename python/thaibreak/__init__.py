"""
ThaiBreak — Fast, high-accuracy Thai word segmentation and typographic line breaker.
"""

from .core import init, words, lines, wrap, display_width

__all__ = ["init", "words", "lines", "wrap", "display_width"]
__version__ = "1.0.0"
