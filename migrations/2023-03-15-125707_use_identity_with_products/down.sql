ALTER TABLE products
DROP COLUMN id;
ALTER TABLE products
ADD COLUMN id SERIAL PRIMARY KEY;