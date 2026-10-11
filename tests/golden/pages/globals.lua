-- Global element functions: replace, delete, splice, constructors.
function Str(el)
    if el.text == "world" then
        return pandoc.Emph("planet")
    end
end

function Code(el)
    el.text = el.text:upper()
    return el
end

function Link(el)
    el.target = el.target .. "/path"
    return el
end

function RawBlock(el)
    if el.text:match("remove me") then
        return {}
    end
end

function Para(el)
    if el.content[1].text == "Para" then
        return { pandoc.Para({ pandoc.Str("First"), pandoc.Space(), pandoc.Str("half.") }),
                 pandoc.Para("Second half " .. FORMAT .. ".") }
    end
end

function Header(el)
    el.level = el.level + 1
    return el
end

function Meta(meta)
    meta.title = pandoc.Strong("Changed")
    return meta
end
