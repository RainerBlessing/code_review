{ pkgs ? import <nixpkgs> {} }:

pkgs.stdenv.mkDerivation {
  name = "example-with-openssl";
  
  src = ./.;

  buildInputs = [
    pkgs.openssl
    pkgs.pkg-config # Fügt pkg-config hinzu, damit OpenSSL gefunden wird
  ];

  # Optional: Add any other dependencies or build steps
  # Example: buildPhase, installPhase

  meta = {
    description = "Example project with OpenSSL as dependency";
    license = pkgs.lib.licenses.mit;
    maintainers = [ pkgs.lib.maintainers.yourname ];
  };

shellHook = ''
    if [ -f ~/.profile ]; then
      source ~/.profile
    fi

    cargo build

    echo "Virtuelle Umgebung und Abhängigkeiten wurden eingerichtet!"
  '';
}

