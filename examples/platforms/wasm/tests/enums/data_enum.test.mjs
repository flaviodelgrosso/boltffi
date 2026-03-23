import { assert, demo } from "../support/index.mjs";

export async function run() {
  const circle = demo.makeCircle(5);
  assert.equal(circle.tag, "Circle");
  assert.equal(circle.radius, 5);

  const rectangle = demo.makeRectangle(3, 4);
  assert.equal(rectangle.tag, "Rectangle");
  assert.equal(rectangle.width, 3);
  assert.equal(rectangle.height, 4);

  assert.deepEqual(demo.echoShape(demo.makeCircle(2)), demo.makeCircle(2));
  assert.deepEqual(demo.echoShape(demo.makeRectangle(3, 4)), demo.makeRectangle(3, 4));
  assert.equal(demo.echoVecShape([demo.makeCircle(2), demo.makeRectangle(3, 4), { tag: "Point" }]).length, 3);

  const textMessage = { tag: "Text", body: "hello" };
  const imageMessage = { tag: "Image", url: "https://example.com/image.png", width: 640, height: 480 };
  assert.deepEqual(demo.echoMessage(textMessage), textMessage);
  assert.deepEqual(demo.echoMessage(imageMessage), imageMessage);
  assert.equal(demo.messageSummary({ tag: "Text", body: "hi" }), "text: hi");
  assert.equal(
    demo.messageSummary(imageMessage),
    "image: 640x480 at https://example.com/image.png",
  );
  assert.equal(demo.messageSummary({ tag: "Ping" }), "ping");

  const dog = { tag: "Dog", name: "Rex", breed: "Labrador" };
  const cat = { tag: "Cat", name: "Milo", indoor: true };
  assert.deepEqual(demo.echoAnimal(dog), dog);
  assert.deepEqual(demo.echoAnimal(cat), cat);
  assert.equal(demo.animalName({ tag: "Fish", count: 5 }), "5 fish");
  assert.equal(demo.animalName(cat), "Milo");
}
