const spawn = require('child_process').spawn;
const path = require('path');
const glob = require("glob");

async function getSources() {
    return new Promise((resolve, reject) => {
        glob("contracts/**/*.sol", function (err, files) {
          if (err) {
              reject(err);
          } else {
              resolve(files);
          }
        });
    })
}

function compile(outdir, path) {
    return new Promise((resolve, reject) => {
        console.log(["-o", outdir, path, "--combined-json", "bin,abi", "--overwrite"])
        const ls = spawn("solc", ["-o", outdir, path, "--combined-json", "bin,abi", "--overwrite"])

        ls.stdout.on('data', (data) => {
            console.log(`stdout: ${data}`);
        });

        ls.stderr.on('data', (data) => {
            console.log(`stderr: ${data}`);
        });

        ls.on('close', (code) => {
            console.log(`child process exited with code ${code}`);
            if (code == 0) {
                resolve(code);
            } else {
                reject(code);
            }
        });
    })
}

async function main() {
    const sourceFilesList = await getSources();
    const sources = sourceFilesList.map(s=>[path.join(path.dirname(s),path.basename(s,".sol")),s]);
    for (const [s,sp] of sources) {
        compile(path.join("newbuild",s),sp);
    }
}

main().then(()=>console.log("done"));
