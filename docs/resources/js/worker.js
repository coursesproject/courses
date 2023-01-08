// load pyodide.js
importScripts("https://cdn.jsdelivr.net/pyodide/v0.20.0/full/pyodide.js");

// Initialize pyodide and load Pandas
async function initialize(){
    self.pyodide = await loadPyodide();
    await self.pyodide.loadPackage("pandas");
}

let initialized = initialize();

self.onmessage = async function(e) {
    await initialized;

    let result = await self.pyodide.runPythonAsync(e.data.src);

    self.postMessage({
        res: result
    });
}