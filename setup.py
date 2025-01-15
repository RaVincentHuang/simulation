from setuptools import setup, find_packages
from setuptools_rust import Binding, RustExtension

setup(
    name="simulation",
    version="0.1.0",
    packages=find_packages(where="python"),
    package_dir={"": "python"},
    rust_extensions=[
        RustExtension(
            "simulation._rust",
            binding=Binding.PyO3,
            debug=False
        )
    ],
    zip_safe=False,
    include_package_data=True,
    python_requires=">=3.10",
    install_requires=[
        "numpy>=1.20.0",
    ],
    extras_require={
        "dev": [
            "pytest>=6.0",
            "pytest-cov>=2.0",
            "black>=22.0",
            "mypy>=0.9",
        ],
    },
)