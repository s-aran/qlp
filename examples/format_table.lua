function dump(o)
    if type(o) == 'table' then
        local s = '{ '
        for k, v in pairs(o) do
            if type(k) ~= 'number' then
                k = '"' .. k .. '"'
            end
            s = s .. '[' .. k .. '] = ' .. dump(v) .. ','
        end
        return s .. '} '
    else
        return tostring(o)
    end
end

print("*** If Japanese characters are garbled, try 'chcp 65001' ***")

print("--------------------------------------------------")
print("*** Clipboard raw text:")
print(qlp.raw)
print("--------------------------------------------------")

-- print(qlp['html'])
-- print(qlp['parsed'])
-- print(qlp['parsed'][1][1].text)

print(dump(qlp.parsed))

qlp.result = "completed!!"
