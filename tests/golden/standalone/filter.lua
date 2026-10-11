-- Pandoc Lua filter. pandoc injects the `pandoc` module, PANDOC_STATE and
-- FORMAT as globals and picks up the filter by the RawBlock/RawInline/Pandoc
-- globals it defines, so those are declared here rather than in the shared
-- fleet-wide .luacheckrc.
-- luacheck: read globals pandoc PANDOC_STATE FORMAT
-- luacheck: globals RawBlock RawInline Pandoc

-- Extract the course path (e.g. "languages/c++") from the input filename.
local function get_course_path()
    local input = PANDOC_STATE.input_files[1] or ""
    local path = input:match("syllabi/courses/(.+)/[^/]+$")
        or input:match("out/processor%.generator%.generic/(.+)/[^/]+$")
    return path
end

-- Build the category line: plain text for pdf/docx, linked breadcrumbs for html.
local function build_category_block(course_path)
    local inlines = {pandoc.Strong({pandoc.Str("Category:")}), pandoc.Space()}
    if FORMAT:match("html") then
        local segments = {}
        for seg in course_path:gmatch("[^/]+") do
            table.insert(segments, seg)
        end
        for i, seg in ipairs(segments) do
            if i > 1 then
                table.insert(inlines, pandoc.Str(" / "))
            end
            -- Build the full folder path: courses/seg1/seg2/...
            local folder_parts = {"courses"}
            for j = 1, i do
                table.insert(folder_parts, segments[j])
            end
            local folder = table.concat(folder_parts, "/")
            -- Use fragment-only URL since the syllabus is viewed inline
            -- within the index page via fetch + innerHTML.
            local url = "#folder=" .. folder
            table.insert(inlines, pandoc.Link({pandoc.Str(seg)}, url))
        end
    else
        table.insert(inlines, pandoc.Str(course_path))
    end
    return pandoc.Para(inlines)
end

-- Strip <!-- chapter: ..., duration: ... --> comments from output.
function RawBlock(el)
    if el.format == "html" and el.text:match("^%s*<!%-%-.*chapter:.*%-%->%s*$") then
        return {}
    end
end

function RawInline(el)
    if el.format == "html" and el.text:match("<!%-%-.*chapter:.*%-%->") then
        return {}
    end
end

-- Set the document title from the first H1 heading if no title is set,
-- remove that H1 to avoid duplication in standalone mode,
-- and insert the course category path.
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
    local course_path = get_course_path()
    if course_path then
        table.insert(doc.blocks, 1, build_category_block(course_path))
    end
    return doc
end
