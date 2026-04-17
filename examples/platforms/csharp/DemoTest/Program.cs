using System;
using static Demo.Demo;

namespace BoltFFI.Demo.Tests;

public static class DemoTest
{
    public static int Main()
    {
        Console.WriteLine("Testing C# bindings...\n");
        TestBool();
        TestI8();
        TestU8();
        TestI16();
        TestU16();
        TestI32();
        TestU32();
        TestI64();
        TestU64();
        TestF32();
        TestF64();
        TestUsize();
        TestIsize();
        TestStrings();
        Console.WriteLine("All tests passed!");
        return 0;
    }

    private static void TestBool()
    {
        Console.WriteLine("Testing bool...");
        Require(EchoBool(true), "echoBool(true)");
        Require(!EchoBool(false), "echoBool(false)");
        Require(!NegateBool(true), "negateBool(true)");
        Require(NegateBool(false), "negateBool(false)");
        Console.WriteLine("  PASS\n");
    }

    private static void TestI8()
    {
        Console.WriteLine("Testing i8...");
        Require(EchoI8(42) == 42, "echoI8(42)");
        Require(EchoI8(-128) == -128, "echoI8(min)");
        Require(EchoI8(127) == 127, "echoI8(max)");
        Console.WriteLine("  PASS\n");
    }

    private static void TestU8()
    {
        Console.WriteLine("Testing u8...");
        Require(EchoU8(0) == 0, "echoU8(0)");
        Require(EchoU8(255) == 255, "echoU8(max)");
        Console.WriteLine("  PASS\n");
    }

    private static void TestI16()
    {
        Console.WriteLine("Testing i16...");
        Require(EchoI16(-32768) == -32768, "echoI16(min)");
        Require(EchoI16(32767) == 32767, "echoI16(max)");
        Console.WriteLine("  PASS\n");
    }

    private static void TestU16()
    {
        Console.WriteLine("Testing u16...");
        Require(EchoU16(0) == 0, "echoU16(0)");
        Require(EchoU16(65535) == 65535, "echoU16(max)");
        Console.WriteLine("  PASS\n");
    }

    private static void TestI32()
    {
        Console.WriteLine("Testing i32...");
        Require(EchoI32(42) == 42, "echoI32(42)");
        Require(EchoI32(-100) == -100, "echoI32(-100)");
        Require(AddI32(10, 20) == 30, "addI32(10, 20)");
        Console.WriteLine("  PASS\n");
    }

    private static void TestU32()
    {
        Console.WriteLine("Testing u32...");
        Require(EchoU32(0u) == 0u, "echoU32(0)");
        Require(EchoU32(uint.MaxValue) == uint.MaxValue, "echoU32(max)");
        Console.WriteLine("  PASS\n");
    }

    private static void TestI64()
    {
        Console.WriteLine("Testing i64...");
        Require(EchoI64(9999999999L) == 9999999999L, "echoI64(large)");
        Require(EchoI64(-9999999999L) == -9999999999L, "echoI64(negative large)");
        Console.WriteLine("  PASS\n");
    }

    private static void TestU64()
    {
        Console.WriteLine("Testing u64...");
        Require(EchoU64(0UL) == 0UL, "echoU64(0)");
        Require(EchoU64(ulong.MaxValue) == ulong.MaxValue, "echoU64(max)");
        Console.WriteLine("  PASS\n");
    }

    private static void TestF32()
    {
        Console.WriteLine("Testing f32...");
        Require(Math.Abs(EchoF32(3.14f) - 3.14f) < 0.001f, "echoF32(3.14)");
        Require(Math.Abs(AddF32(1.5f, 2.5f) - 4.0f) < 0.001f, "addF32(1.5, 2.5)");
        Console.WriteLine("  PASS\n");
    }

    private static void TestF64()
    {
        Console.WriteLine("Testing f64...");
        Require(Math.Abs(EchoF64(3.14159265359) - 3.14159265359) < 0.0000001, "echoF64(pi)");
        Require(Math.Abs(AddF64(1.5, 2.5) - 4.0) < 0.0000001, "addF64(1.5, 2.5)");
        Console.WriteLine("  PASS\n");
    }

    private static void TestUsize()
    {
        Console.WriteLine("Testing usize...");
        Require(EchoUsize((nuint)42) == (nuint)42, "echoUsize(42)");
        Require(EchoUsize((nuint)0) == (nuint)0, "echoUsize(0)");
        Console.WriteLine("  PASS\n");
    }

    private static void TestIsize()
    {
        Console.WriteLine("Testing isize...");
        Require(EchoIsize((nint)42) == (nint)42, "echoIsize(42)");
        Require(EchoIsize((nint)(-100)) == (nint)(-100), "echoIsize(-100)");
        Console.WriteLine("  PASS\n");
    }

    private static void TestStrings()
    {
        Console.WriteLine("Testing strings...");
        Require(EchoString("hello") == "hello", "echoString(hello)");
        Require(EchoString("") == "", "echoString(empty)");
        Require(EchoString("café") == "café", "echoString(unicode)");
        Require(EchoString("日本語") == "日本語", "echoString(cjk)");
        Require(EchoString("hello 🌍 world") == "hello 🌍 world", "echoString(emoji)");

        Require(ConcatStrings("foo", "bar") == "foobar", "concatStrings(foo, bar)");
        Require(ConcatStrings("", "bar") == "bar", "concatStrings(empty, bar)");
        Require(ConcatStrings("foo", "") == "foo", "concatStrings(foo, empty)");
        Require(ConcatStrings("🎉", "🎊") == "🎉🎊", "concatStrings(emoji)");

        Require(StringLength("hello") == 5u, "stringLength(hello)");
        Require(StringLength("") == 0u, "stringLength(empty)");
        Require(StringLength("café") == 5u, "stringLength(utf8 bytes)");
        Require(StringLength("🌍") == 4u, "stringLength(emoji 4 bytes)");

        Require(StringIsEmpty(""), "stringIsEmpty(empty)");
        Require(!StringIsEmpty("x"), "stringIsEmpty(nonempty)");

        Require(RepeatString("ab", 3u) == "ababab", "repeatString(ab, 3)");
        Require(RepeatString("x", 0u) == "", "repeatString(x, 0)");
        Require(RepeatString("🌟", 2u) == "🌟🌟", "repeatString(emoji, 2)");
        Console.WriteLine("  PASS\n");
    }

    private static void Require(bool condition, string label)
    {
        if (!condition) throw new InvalidOperationException($"FAIL: {label}");
    }
}
