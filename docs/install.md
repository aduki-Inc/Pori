# Installation Guide

This guide provides comprehensive instructions for installing Pori across different platforms and hosting your own installation scripts.

## Quick Installation

### Linux/macOS

```bash
curl -fsSL https://raw.githubusercontent.com/aduki-Inc/Pori/main/release/install.sh | bash
```

### Windows (PowerShell)

```powershell
iwr -Uri "https://raw.githubusercontent.com/aduki-Inc/Pori/main/release/install.ps1" -OutFile "install.ps1"; .\install.ps1
```

## Platform-Specific Installation

### Linux

#### Automated Installation (Recommended)

```bash
# Install latest version
curl -fsSL https://raw.githubusercontent.com/aduki-Inc/Pori/main/release/install.sh | bash

# Install specific version
curl -fsSL https://raw.githubusercontent.com/aduki-Inc/Pori/main/release/install.sh | bash -s -- --version v0.1.4

# Install to custom directory
curl -fsSL https://raw.githubusercontent.com/aduki-Inc/Pori/main/release/install.sh | bash -s -- --dir ~/.local/bin
```

#### Manual Installation

```bash
# Download the binary
wget https://github.com/aduki-Inc/Pori/releases/latest/download/pori-linux-x86_64.tar.gz

# Extract
tar -xzf pori-linux-x86_64.tar.gz

# Make executable and move to PATH
chmod +x pori
sudo mv pori /usr/local/bin/

# Verify installation
pori --version
```

#### Adding to PATH

Add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
export PATH="/usr/local/bin:$PATH"
```

### macOS

#### Automated Installation (Recommended)

```bash
# Install latest version
curl -fsSL https://raw.githubusercontent.com/aduki-Inc/Pori/main/release/install.sh | bash

# For Apple Silicon Macs, the script will automatically detect and download the ARM64 version
# For Intel Macs, it will download the x86_64 version
```

#### Manual Installation

```bash
# For Intel Macs
curl -L https://github.com/aduki-Inc/Pori/releases/latest/download/pori-darwin-x86_64.tar.gz -o pori.tar.gz

# For Apple Silicon Macs
curl -L https://github.com/aduki-Inc/Pori/releases/latest/download/pori-darwin-aarch64.tar.gz -o pori.tar.gz

# Extract and install
tar -xzf pori.tar.gz
chmod +x pori
sudo mv pori /usr/local/bin/

# Verify installation
pori --version
```

#### Adding to PATH

Add to your shell profile (`~/.bash_profile`, `~/.zshrc`, etc.):

```bash
export PATH="/usr/local/bin:$PATH"
```

### Windows

#### Automated Installation (PowerShell)

```powershell
# Download and run the installation script
iwr -Uri "https://raw.githubusercontent.com/aduki-Inc/Pori/main/release/install.ps1" -OutFile "install.ps1"
.\install.ps1

# Or install specific version
.\install.ps1 -Version v0.1.4

# Or install to custom directory
.\install.ps1 -InstallDir "C:\Tools\Pori"
```

#### Manual Installation

```powershell
# Download the binary
Invoke-WebRequest -Uri "https://github.com/aduki-Inc/Pori/releases/latest/download/pori-windows-x86_64.zip" -OutFile "pori.zip"

# Extract
Expand-Archive -Path "pori.zip" -DestinationPath "pori"

# Move to a directory in PATH
mkdir "C:\Tools\Pori"
Move-Item "pori\pori.exe" "C:\Tools\Pori\"

# Add to PATH
$env:Path += ";C:\Tools\Pori"
[Environment]::SetEnvironmentVariable("Path", $env:Path, "User")

# Verify installation
pori --version
```

## Hosting Your Own Installation Scripts

### Setting Up GitHub for Script Hosting

1. **Ensure scripts are in the repository:**

   ``` bash
   release/
   ├── install.sh        # Linux/macOS installation script
   ├── install.ps1       # Windows PowerShell script
   └── README.md         # Installation documentation
   ```

2. **Make scripts executable and accessible:**
   - Ensure `install.sh` has executable permissions in the repository
   - Host scripts in the `main` branch for stability
   - Use raw GitHub URLs for direct script access

3. **Raw GitHub URLs format:**

   ``` bash
   https://raw.githubusercontent.com/{owner}/{repo}/{branch}/path/to/script
   ```

### Installation Script Requirements

#### For Linux/macOS (`install.sh`)

The script should:

- Detect platform and architecture automatically
- Download the appropriate binary for the platform
- Install to a directory in PATH (`/usr/local/bin` by default)
- Handle permissions (sudo when needed)
- Verify installation
- Support custom installation directories
- Support specific version installation
- Provide helpful error messages

#### For Windows (`install.ps1`)

The script should:

- Detect Windows architecture (x86_64)
- Download and extract the Windows binary
- Install to a user-accessible location
- Automatically add to user PATH
- Verify installation
- Handle PowerShell execution policy

### Advanced Installation Options

#### Linux Package Managers

**For Debian/Ubuntu (APT):**

Create a repository structure:

```bash
# Add GPG key and repository
curl -fsSL https://packages.example.com/pori/gpg | sudo apt-key add -
echo "deb https://packages.example.com/pori/apt stable main" | sudo tee /etc/apt/sources.list.d/pori.list

# Install
sudo apt update
sudo apt install pori
```

**For Red Hat/CentOS (YUM/DNF):**

```bash
# Add repository
sudo yum-config-manager --add-repo https://packages.example.com/pori/rpm/pori.repo

# Install
sudo yum install pori
```

#### macOS Package Managers

**Homebrew Formula:**

Create a formula for Homebrew:

```ruby
class Pori < Formula
  desc "Pori tunnel client"
  homepage "https://github.com/aduki-Inc/Pori"
  url "https://github.com/aduki-Inc/Pori/releases/download/v0.1.4/pori-darwin-x86_64.tar.gz"
  sha256 "a2d628b7c937e490f819ad7da3ea674d77020e27e2583331643f14c323c04f88"
  version "0.1.4"

  def install
    bin.install "pori"
  end

  test do
    system "#{bin}/pori", "--version"
  end
end
```

#### Windows Package Managers

**Chocolatey Package:**

Create a chocolatey package specification:

```xml
<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
  <metadata>
    <id>pori</id>
    <version>0.1.4</version>
    <title>Pori</title>
    <authors>aduki-Inc</authors>
    <description>Pori tunnel client for secure remote access</description>
    <projectUrl>https://github.com/aduki-Inc/Pori</projectUrl>
    <tags>tunnel proxy websocket</tags>
  </metadata>
</package>
```

**Winget Package:**

Submit to the Windows Package Manager repository.

### Verification and Security

#### Checksum Verification

Always provide checksums for downloads:

```bash
# Generate checksums (done automatically by CI)
sha256sum pori-*.tar.gz pori-*.zip > SHA256SUMS

# Verify downloads
sha256sum -c SHA256SUMS
```

#### GPG Signing

For enhanced security, sign releases:

```bash
# Sign the release
gpg --armor --detach-sign SHA256SUMS

# Verify signature
gpg --verify SHA256SUMS.asc SHA256SUMS
```

## Troubleshooting

### Common Issues

1. **Permission Denied**

   ```bash
   # Solution: Use sudo or install to user directory
   curl -fsSL https://raw.githubusercontent.com/aduki-Inc/Pori/main/release/install.sh | bash -s -- --dir ~/.local/bin
   ```

2. **Binary Not in PATH**

   ```bash
   # Add to shell profile
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

3. **Windows Execution Policy**

   ```powershell
   # Allow script execution
   Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
   ```

4. **Architecture Mismatch**
   - Ensure you're downloading the correct binary for your platform
   - Use the automated installation script for automatic detection

### Manual PATH Configuration

#### Linux/macOS

Add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
# For system-wide installation
export PATH="/usr/local/bin:$PATH"

# For user installation
export PATH="$HOME/.local/bin:$PATH"
```

#### Windows

Using PowerShell (run as Administrator for system-wide):

```powershell
# User PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
[Environment]::SetEnvironmentVariable("Path", "$userPath;C:\Tools\Pori", "User")

# System PATH (requires Administrator)
$systemPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
[Environment]::SetEnvironmentVariable("Path", "$systemPath;C:\Tools\Pori", "Machine")
```

## URL Installation Examples

### For Your Repository (aduki-Inc/Pori)

**Linux/macOS One-liner:**

```bash
curl -fsSL https://raw.githubusercontent.com/aduki-Inc/Pori/main/release/install.sh | bash
```

**Windows One-liner:**

```powershell
iwr -Uri "https://raw.githubusercontent.com/aduki-Inc/Pori/main/release/install.ps1" -OutFile "install.ps1"; .\install.ps1
```

### For Custom Hosting

If you want to host on your own domain:

1. **Upload scripts to your web server:**

   ```bash
   https://yourdomain.com/install.sh      # Linux/macOS
   https://yourdomain.com/install.ps1     # Windows
   ```

2. **Update installation commands:**

   ```bash
   # Linux/macOS
   curl -fsSL https://yourdomain.com/install.sh | bash
   
   # Windows
   iwr -Uri "https://yourdomain.com/install.ps1" -OutFile "install.ps1"; .\install.ps1
   ```

## Support

For installation issues:

1. Check the [GitHub Issues](https://github.com/aduki-Inc/Pori/issues)
2. Review the [Documentation](https://github.com/aduki-Inc/Pori/tree/main/docs)
3. Create a new issue with:
   - Operating system and version
   - Architecture (x86_64, ARM64, etc.)
   - Installation method attempted
   - Complete error messages

## Quick Reference

| Platform | Architecture | Command |
|----------|--------------|---------|
| Linux | x86_64/ARM64 | `curl -fsSL https://raw.githubusercontent.com/aduki-Inc/Pori/main/release/install.sh \| bash` |
| macOS | Intel/Apple Silicon | `curl -fsSL https://raw.githubusercontent.com/aduki-Inc/Pori/main/release/install.sh \| bash` |
| Windows | x86_64 | `iwr -Uri "https://raw.githubusercontent.com/aduki-Inc/Pori/main/release/install.ps1" -OutFile "install.ps1"; .\install.ps1` |

All installation methods automatically:

- Detect your platform and architecture
- Download the correct binary
- Install to an appropriate location
- Add the binary to your PATH
- Verify the installation works
