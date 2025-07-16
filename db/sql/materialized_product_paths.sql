
CREATE MATERIALIZED VIEW IF NOT EXISTS product_paths AS
WITH relative_depths AS (
    SELECT 
        h1.ancestor,
        h1.descendant,
        h2.depth AS ancestor_depth
    FROM product_categories_hierarchy h1
    LEFT JOIN product_categories_hierarchy h2
        ON h1.ancestor = h2.ancestor AND h1.ancestor = h2.descendant
)
SELECT 
    rd.descendant AS id,
    ARRAY_AGG(c.name ORDER BY rd.ancestor_depth) AS names,
    ARRAY_AGG(c.id ORDER BY rd.ancestor_depth) AS ids
FROM relative_depths rd
JOIN product_categories c ON c.id = rd.ancestor
GROUP BY rd.descendant;
