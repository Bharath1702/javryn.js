// 05_classes.js — OOP & Inheritance
class Animal {
    constructor(name) {
        this.name = name;
    }
    speak() {
        return `${this.name} makes a sound`;
    }
}

class Dog extends Animal {
    speak() {
        return `${this.name} barks`;
    }
}

const d = new Dog("Rex");
console.log(d.speak());
