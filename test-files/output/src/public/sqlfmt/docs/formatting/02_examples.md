---
title: "Examples"
---

# Examples

A tour of how SQLfmt reshapes common statements. Each example shows the input
you might pipe in and the formatted output SQLfmt returns.

## SELECT with JOIN

**Input**

```sql
select u.name, o.total from users u inner join orders o on u.id = o.user_id where o.total > 100;
```

**Output**

```sql
SELECT u.name, o.total FROM users u INNER JOIN orders o ON u.id = o.user_id WHERE o.total > 100;
```

## SELECT with subquery

**Input**

```sql
select id, (select max(price) from orders o where o.user_id = u.id) as max_order from users u where u.active = 1;
```

**Output**

```sql
SELECT id, (
    SELECT max(price) FROM orders o WHERE o.user_id = u.id
) AS max_order FROM users u WHERE u.active = 1;
```

## CREATE TABLE

**Input**

```sql
create table users (id bigint unsigned not null auto_increment primary key, name varchar(255) not null, email varchar(255) not null unique, created_at timestamp default current_timestamp);
```

**Output**

```sql
CREATE TABLE users (
    id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    name       VARCHAR(255)    NOT NULL,
    email      VARCHAR(255)    NOT NULL UNIQUE,
    created_at TIMESTAMP       DEFAULT CURRENT_TIMESTAMP
);
```

## CREATE VIEW

**Input**

```sql
create view active_users as select u.id as user_id, u.name as user_name, o.total from users u inner join orders o on u.id = o.user_id where o.total > 100;
```

**Output**

```sql
CREATE VIEW active_users AS
SELECT
    u.id   AS user_id,
    u.name AS user_name,
    o.total
FROM users u
    INNER JOIN orders o
        ON u.id = o.user_id
WHERE o.total > 100;
```

## UPDATE

**Input**

```sql
update users set name = 'Alice', email = 'alice@example.com' where id = 1;
```

**Output**

```sql
UPDATE users
SET
    name = 'Alice',
    email = 'alice@example.com'
WHERE id = 1;
```

## DELETE

**Input**

```sql
delete from users where id = 1;
```

**Output**

```sql
DELETE FROM users
WHERE id = 1;
```

## INSERT (long — wraps when over 100 characters)

**Input**

```sql
insert into users (id, name, email, status, role, department) values (1, 'Alexander Hamilton', 'alex@example.com', 'active', 'admin', 'engineering'), (2, 'Benjamin Franklin', 'ben@example.com', 'active', 'user', 'marketing');
```

**Output**

```sql
INSERT INTO users (id, name, email, status, role, department) VALUES
(1,'Alexander Hamilton','alex@example.com','active','admin','engineering'),
(2,'Benjamin Franklin','ben@example.com','active','user','marketing');
```

## Operators

**Input**

```sql
select * from users where age >= 18 and age <= 65 and name != 'admin' and status <> 'inactive';
```

**Output**

```sql
SELECT * FROM users WHERE age >= 18 AND age <= 65 AND name != 'admin' AND status != 'inactive';
```
