Back:
- SurrealDB v
- CRUD v
- разобраться с JOIN v
```
SELECT *,
(SELECT * FROM artefacts WHERE crafted_item_id = 'T4_2H_CLEAVER_HELL') AS artefact,
(SELECT * FROM items WHERE crafted_item_ids CONTAINS 'T4_2H_CLEAVER_HELL') AS resource
FROM items WHERE item_id = 'T4_2H_CLEAVER_HELL';

1. добавляю таблицу artefacts
2. у каждого артефакта должен быть crafted_item_id = item_id
3. добавляю таблицу resources
4. у каждого ресурса должен быть crafted_item_id = item_id
5. убираю из items поле craft_price - оно будет высчитываться на лету
6. кол-во ресурсов должно лежать в items: [{ ...item, resources: { item_id, quantity }}]
7. тогда на фронте зная кол-во ресурса и получив инфу по ресурсам из resources 
я смогу высчитать себестоимость
```

- Авторизация
- Guard
- Сущности users
- Сущности orders
- Сущности statuses
- Сущности events
- role based access control
- tests
- permissions
- связка пользователей и филиалов
  -- у пользователя есть массив филиалов к которым он относится и он может видеть сущности только в рамках своего филиала (пока только для пользователей)
- websockets
- messages
- запрашивать события за месяц

Client:
- Форма авторизации
- Форма логина
- единый layout
- role based access control
- collapsed sidebar
- dashboard
- users
- settings
- dark mode
- events
- calendar
- toasts
- tests
- permissions
- websockets
- chat
