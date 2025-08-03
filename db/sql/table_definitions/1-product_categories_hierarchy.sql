CREATE TABLE IF NOT EXISTS product_categories_hierarchy(
    ancestor integer NOT NULL,
    descendant integer NOT NULL,
    depth integer NOT NULL,
    PRIMARY KEY(ancestor,descendant),
    CONSTRAINT product_category_hierarchy_ancestor_fkey FOREIGN key(ancestor) REFERENCES product_categories(id),
    CONSTRAINT product_category_hierarchy_decendant_fkey FOREIGN key(descendant) REFERENCES product_categories(id)
);