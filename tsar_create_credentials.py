#!/usr/bin/env python3
import click
import subprocess


@click.command()
@click.option("--cache_dir", type=str, required=True, help="directory to cache authentication and settings files")
@click.option("--librespot_binary", type=str, default="librespot", help="path to the librespot binary")
def main(cache_dir, librespot_binary):

    # setup recorder


    command = [librespot_binary, "--system-cache", cache_dir, "--enable-oauth"] 
    print("starting recorder with command: ")
    print(command)
    subprocess.run(command, shell=False)


if __name__ == "__main__":
    main()
