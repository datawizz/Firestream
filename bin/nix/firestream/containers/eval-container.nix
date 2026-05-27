# Container Evaluation Mechanism
# Copyright Firestream. MIT License.
#
# This module provides a uniform, options-driven entrypoint for evaluating a
# single container. It declares a standard option schema under the container's
# name, evaluates it together with the container's own options modules, then
# dispatches to the appropriate firestream container factory based on
# runtimeType.
#
# It is NOT wired into the flake outputs in Phase 1 - it is introduced here so
# later phases can migrate containers onto a shared, declarative options schema.
#
# Usage (later phases):
#   let
#     result = (import ./eval-container.nix { inherit pkgs lib firestreamLib; }) {
#       name = "airflow";
#       runtimeType = "python-workspace";
#       workspacePath = ./airflow;
#       modules = [ ./airflow/options.nix ];
#     };
#   in result.dockerImage

{ pkgs, lib, firestreamLib }:

{
  name,
  runtimeType,
  modules ? [],
  workspacePath ? null,
  modulePath ? null,
  extraFactoryArgs ? {},
}:

let
  # Standard option schema declared under the ${name} namespace.
  # The container's own options.nix supplies the concrete `version`.
  mkContainerOptions = { name }: { lib, ... }: {
    options.${name} = {
      version = lib.mkOption {
        type = lib.types.str;
        description = "Container version (supplied by the container's options.nix).";
      };

      python = lib.mkOption {
        type = lib.types.nullOr lib.types.package;
        default = null;
        description = "Python interpreter package (python-workspace runtime).";
      };

      paths = lib.mkOption {
        type = lib.types.attrsOf lib.types.str;
        default = {};
        description = "Filesystem path overrides for the container.";
      };

      env = lib.mkOption {
        type = lib.types.attrsOf lib.types.str;
        default = {};
        description = "Environment variables to bake into the container.";
      };

      envSecrets = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [];
        description = "Names of environment variables sourced from secrets at runtime.";
      };

      exposedPorts = lib.mkOption {
        type = lib.types.listOf lib.types.int;
        default = [];
        description = "TCP ports the container exposes.";
      };

      image = lib.mkOption {
        default = {};
        type = lib.types.submodule {
          options = {
            registry = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              default = null;
              description = "Optional image registry. When null, no registry is prepended.";
            };
            repository = lib.mkOption {
              type = lib.types.str;
              default = "firestream-${name}";
              description = "Image repository name.";
            };
            tag = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              default = null;
              description = "Optional image tag. When null, falls back to version.";
            };
          };
        };
        description = "Docker image naming configuration.";
      };

      extraModuleArgs = lib.mkOption {
        type = lib.types.attrsOf lib.types.raw;
        default = {};
        description = "Additional arguments forwarded to the container factory/module.";
      };
    };
  };

  evaled = lib.evalModules {
    modules = [ (mkContainerOptions { inherit name; }) ] ++ modules;
    specialArgs = { inherit pkgs lib; };
  };

  cfg = evaled.config.${name};

  imageTag = if cfg.image.tag != null then cfg.image.tag else cfg.version;

  imageName =
    if cfg.image.registry != null
    then "${cfg.image.registry}/${cfg.image.repository}"
    else cfg.image.repository;

  # Load the container's Python wheel/source build overrides, mirroring the
  # legacy flake which passed `overrides = import .../overrides.nix { pkgs; lib; }`
  # into mkPythonWorkspaceContainer. Every python-workspace container ships an
  # overrides.nix in its workspace; without these the Python closure differs
  # (and some packages fail to build). The { pkgs, lib } call contract is
  # unchanged from the legacy flake.
  overrides =
    if workspacePath != null && builtins.pathExists (workspacePath + "/overrides.nix")
    then import (workspacePath + "/overrides.nix") { inherit pkgs lib; }
    else {};

  module =
    if runtimeType == "python-workspace" then
      firestreamLib.mkPythonWorkspaceContainer ({
        inherit workspacePath name;
        version = cfg.version;
        python = cfg.python;
        inherit overrides;
        moduleArgs = {
          envVars = cfg.env;
          envVarsWithSecrets = cfg.envSecrets;
          paths = cfg.paths;
          exposedPorts = cfg.exposedPorts;
          inherit imageName imageTag;
        } // cfg.extraModuleArgs;
      } // extraFactoryArgs)
    else if runtimeType == "system" || runtimeType == "java" then
      import modulePath ({
        inherit pkgs lib;
        firestream = firestreamLib;
        version = cfg.version;
        envVars = cfg.env;
        envVarsWithSecrets = cfg.envSecrets;
        paths = cfg.paths;
        exposedPorts = cfg.exposedPorts;
        inherit imageName imageTag;
      } // extraFactoryArgs // cfg.extraModuleArgs)
    else
      throw "eval-container: unknown runtimeType ${runtimeType}";

in {
  dockerImage = module.dockerImage;
  metadata = module.metadata or null;
  module = module;
  config = evaled.config;
  options = evaled.options;
}
