local result = exec("curl", {"-XGET", "https://httpbin.org/get"})
qlp.result = result.stdout
