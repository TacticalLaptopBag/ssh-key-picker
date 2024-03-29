#!/bin/lua5.3
local argparse = require("argparse")
local path = require("pl.path")
local dir = require("pl.dir")
require("pl.stringx").import()

local parser = argparse("ssh-key-picker", "Enables ar disables ssh keys by moving them into a directory")
parser:option("-k --key", "Specifies the name of the key to enable.")
local args = parser:parse()

local sshDir = path.expanduser("~/.ssh")
local sshDisabledDir = path.join(sshDir, "disabled-keys")


local function filterPublicKeys(fileNameArray)
    local publicKeys = {}
    for _, v in ipairs(fileNameArray) do
        if v:endswith(".pub") then
            table.insert(publicKeys, path.basename(v))
        end
    end
    return publicKeys
end

local function filterPrivateKeys(fileNameArray)
    local publicKeys = {}
    for _, v in pairs(fileNameArray) do
        if not v:endswith(".pub") then
            table.insert(publicKeys, path.basename(v))
        end
    end
    return publicKeys
end

local function promptKey()
    local keys = filterPrivateKeys(dir.getfiles(sshDisabledDir))
    if #keys == 0 then
        print("No keys found!")
        os.exit(1)
    end

    for i, v in ipairs(keys) do
        print("["..i.."]: "..v)
    end

    local choice = -1
    while choice < 1 or choice > #keys do
        io.write("Choose a key to enable from the list: ")
        choice = tonumber(io.read())

        if choice == nil or math.floor(choice) ~= choice then
            print("Choice must be an integer!")
            choice = -1
        elseif choice < 1 then
            print("Choice cannot be less than 1")
        elseif choice > #keys then
            print("Choice cannot be greater than "..#keys)
        end
    end

    args.key = keys[choice]
end


local publicKeys = filterPublicKeys(dir.getfiles(sshDir))
dir.makepath(sshDisabledDir)

for _, publicKey in ipairs(publicKeys) do
    local privateKey = path.splitext(publicKey)
    local publicKeyPath = path.join(sshDir, publicKey)
    local privateKeyPath = path.join(sshDir, privateKey)

    os.rename(publicKeyPath, path.join(sshDisabledDir, publicKey))
    os.rename(privateKeyPath, path.join(sshDisabledDir, privateKey))
end

if args.key == nil then
    promptKey()
end

if not path.exists(path.join(sshDisabledDir, args.key)) then
    -- Try to find a matching key
    local matches = 0
    local match = ""

    for _, keyPath in ipairs(dir.getfiles(sshDisabledDir)) do
        local key = path.basename(keyPath)
        if key:strip():lower():find(args.key:lower()) then
            match = path.splitext(key)
            matches = matches + 1
        end
    end

    -- 2 matches, because private and public key
    if matches ~= 2 then
        print("Unable to find key \""..args.key.."\"")
        promptKey()
    else
        args.key = match
    end
end

local enabledKey = ""
for _, keyPath in ipairs(dir.getfiles(sshDisabledDir)) do
    local key = path.basename(keyPath)
    local keyName = path.splitext(key)
    if keyName:lower() == args.key:lower() then
        local keyPath = path.join(sshDisabledDir, key)
        os.rename(keyPath, path.join(sshDir, key))
        enabledKey = keyName
    end
end

print("Enabled "..enabledKey)
