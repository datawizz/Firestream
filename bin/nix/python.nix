{
  pkgs,
  lib,
  config,
  ...
}:
{
  environment.systemPackages = with pkgs; [
    # Python 3.12 and core development tools
    python312
    python312Packages.uv
    ty
    python312Packages.black
    # python312Packages.macfsevents

    # Additional tools
    python312Packages.pip
    python312Packages.virtualenv
    # python312Packages.poetry
    python312Packages.ipython
    python312Packages.pytest
    python312Packages.pylint
    python312Packages.mypy
    python312Packages.flake8
    python312Packages.jupyter
  ];

  # Environment variables for Python development
  environment.variables = {
    PYTHONPATH = lib.makeSearchPath "lib/python3.12/site-packages" [
      pkgs.python312
      pkgs.python312Packages.pip
    ];
    PYTHON = "${pkgs.python312}/bin/python";
  };
}
