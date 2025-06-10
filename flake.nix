{
	description = "xd";
	inputs = {
		advisory-db = {
			url = "github:rustsec/advisory-db";
			flake = false;
		};
		fenix = {
			url = "github:nix-community/fenix";
			# inputs.nixpkgs.follows = "nixpkgs";
		};
		crane = {
			url = "github:ipetkov/crane";
			inputs.nixpkgs.follows = "nixpkgs";
			# inputs.flake-utils.follows = "flake-utils";
		};
		nixpkgs.url = "github:NixOs/nixpkgs/nixos-unstable";
		flake-utils.url = "github:numtide/flake-utils";
	};

	outputs = {self, advisory-db, fenix, crane, nixpkgs, flake-utils }: 
	flake-utils.lib.eachDefaultSystem (system:
		let 
			pkgs = nixpkgs.legacyPackages.${system};
			craneLib = (crane.mkLib pkgs).overrideToolchain
				fenix.packages.${system}.stable.toolchain;
			src = craneLib.cleanCargoSource ./.;
			commonArgs = {
				inherit src;
				strictDeps = true;
			};
			cargoArtifacts = craneLib.buildDepsOnly commonArgs;
			commonWithArtifacts = commonArgs // { inherit cargoArtifacts; };
			builtCrate = craneLib.buildPackage commonWithArtifacts;
		in {
			checks = {
				inherit builtCrate;
				crateClippy = craneLib.cargoClippy (
					commonWithArtifacts // {
						cargoClippyExtraArgs = "--all-targets -- --deny warnings";
					}
				);
				crateDocs = craneLib.cargoDoc commonWithArtifacts;
				crateFmt = craneLib.cargoFmt {
					inherit src;
				};
				crateAudit = craneLib.cargoAudit {
					inherit src advisory-db;
				};
			};
			packages.default = builtCrate;
			apps.default = (flake-utils.lib.mkApp {
				drv = builtCrate;
			}) // {
			};
			devShells.default = craneLib.devShell {
				checks = self.checks.${system};
			};
		}
	);
}
