const { Binary } = require("binary-install");

function getBinary() {
  const version = require("../package.json").version;
  const url = `https://github.com/TeamSparta-Inc/teamsparta-cli/releases/download/v${version}/sprt.tar.gz`;
  const name = "sprt";

  return new Binary(name, url);
}

module.exports = getBinary;
