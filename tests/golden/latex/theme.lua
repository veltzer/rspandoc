-- What the fleet's filter does for LaTeX: the title from the first
-- header, a paragraph added in front, and a raw LaTeX block appended to
-- header-includes (which may be absent, one value, or a list).
function Pandoc(doc)
    if not doc.meta.title then
        for i, block in ipairs(doc.blocks) do
            if block.t == "Header" and block.level == 1 then
                doc.meta.title = block.content
                table.remove(doc.blocks, i)
                break
            end
        end
    end
    table.insert(doc.blocks, 1, pandoc.Para({pandoc.Strong({pandoc.Str("Format:")}), pandoc.Space(), pandoc.Str(FORMAT)}))
    if FORMAT == "latex" then
        local f = assert(io.open("preamble.tex", "r"))
        local tex = f:read("a")
        f:close()
        local includes = doc.meta["header-includes"]
        if includes == nil then
            includes = pandoc.MetaList({})
        elseif pandoc.utils.type(includes) ~= "List" then
            includes = pandoc.MetaList({includes})
        end
        includes:insert(pandoc.MetaBlocks({pandoc.RawBlock("latex", tex)}))
        doc.meta["header-includes"] = includes
    end
    return doc
end
