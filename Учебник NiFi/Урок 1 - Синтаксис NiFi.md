## Создание объекта

```
object button{
	id: play_button;
	style: main;
	layer: false;
	
	location{
		up: 50%;
		down: 50%;
		right: 50%;
		left: 50%;
	}
}
```

    object - обозначение объекта
    button - вид объекта
    id - спецаильное id для обращения с бекэндом
	style - стиль обьекта ( привязка к стилю )
    layer - наложение на объект ( false - нельзя \ true - можно)
    location - место нахождения объекта

---

## Стили объектов:

```
style main{
	background-color: #ffffff;
	stroke-color: #000000
}
```

	style - создание стиля для объекта
	main - название стиля
	background-color - цвет фона
	stroke-color - цвет обводки

---
