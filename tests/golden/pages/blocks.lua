-- Block elements beyond paragraphs: ordered lists, code blocks and rules,
-- read, changed and built.
function OrderedList(el)
    local la = el.listAttributes
    if la.delimiter == "OneParen" then
        return pandoc.BulletList(el.content)
    end
    la.start = 4
    la.style = "LowerRoman"
    el.listAttributes = la
    return el
end

function CodeBlock(el)
    if el.text == "fenced" then
        local attr = el.attr
        attr.identifier = "rebuilt"
        return pandoc.CodeBlock("rebuilt <x>", attr)
    end
    el.text = el.text:upper() .. "\n  " .. FORMAT
    return el
end

function HorizontalRule(el)
    return { el, pandoc.OrderedList({ { pandoc.Plain("made") } }), pandoc.HorizontalRule() }
end
