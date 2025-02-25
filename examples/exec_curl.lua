local result = exec("curl", {"-XGET", "https://httpbin.org/get", "-H", "Accept: application/json", "-H",
                             "Authorization: Bearer blahblahblah"})
qlp.result = result.stdout
