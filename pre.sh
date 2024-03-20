#!/bin/sh

apt-get update
apt-get install -y dotnet
dotnet tool install -g Microsoft.Web.LibraryManager.Cli