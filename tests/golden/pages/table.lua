-- A returned list of filters, applied in order.
local count = 0
return {
    {
        Str = function(el)
            count = count + 1
            return el
        end,
    },
    {
        Pandoc = function(doc)
            local file = PANDOC_STATE.input_files[1]
            table.insert(doc.blocks, pandoc.Para(file .. " has words"))
            table.insert(doc.blocks, 1, pandoc.Header(2, "Added"))
            return doc
        end,
    },
    {
        Strong = function(el)
            return el.content
        end,
        RawInline = function(el)
            return pandoc.Str("raw")
        end,
    },
}
