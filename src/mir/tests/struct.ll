; ModuleID = 'test'
source_filename = "test"

%Test = type { %Inner }
%Inner = type { i32 }

define i32 @test() {
  %_0 = alloca i32, align 4
  %_1 = alloca %Test, align 8
  br label %bb0

bb0:                                              ; preds = %0
  %1 = load i32, ptr %_0, align 4
  ret i32 %1
}

!llvm.module.flags = !{!0}
!llvm.dbg.cu = !{!1}

!0 = !{i32 2, !"Debug Info Version", i32 3}
!1 = distinct !DICompileUnit(language: DW_LANG_Rust, file: !2, producer: "ppl", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, sysroot: "/")
!2 = !DIFile(filename: "test", directory: ".")
