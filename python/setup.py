import os
import platform
import sys

from setuptools import Distribution, setup

try:
    from setuptools.command.bdist_wheel import bdist_wheel
except ImportError:  # setuptools < 70.1
    from wheel.bdist_wheel import bdist_wheel


class BinaryDistribution(Distribution):
    """Report native content so the files are installed into platlib."""

    def has_ext_modules(self):
        return True


class PlatformWheel(bdist_wheel):
    """Tag the wheel py3-none-<platform>: it bundles a native library loaded via ctypes."""

    def finalize_options(self):
        super().finalize_options()
        self.root_is_pure = False

    def get_tag(self):
        _, _, plat = super().get_tag()
        if sys.platform == "darwin":
            # The bundled library is single-arch, so never tag the wheel universal2
            target = os.environ.get("MACOSX_DEPLOYMENT_TARGET", "11.0").replace(".", "_")
            plat = f"macosx_{target}_{platform.machine()}"
        return "py3", "none", plat


setup(cmdclass={"bdist_wheel": PlatformWheel}, distclass=BinaryDistribution)
