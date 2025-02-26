local url = qlp.text

local owner, repos, pull = string.match(url, "^https://github.com/([-0-9a-z./_]+)/([-0-9a-z./-]+)/pull/(%d+)$")

local a = "https://api.github.com/repos/" .. owner .. "/" .. repos .. "/pulls/" .. pull

local token = "<your token...>"

local header_auth = "Authorization: Bearer " .. token
local header_accept = "Accept: application/vnd.github+json"

local result = exec("curl", {"-L", "-H", header_accept, "-H", header_auth, a})
local table = json_to_table(result.stdout)

-- print("[" .. table.title .. "](" .. table.url .. ")")

qlp.result = "[" .. table.title .. "](" .. table.url .. ")"

