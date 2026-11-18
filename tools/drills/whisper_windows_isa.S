/* Windows x64 ABI. The drill loads this DLL inside the worker's Python
 * process. Each negative control executes exactly one newer instruction. */
.intel_syntax noprefix
.text

.globl read_cpu
read_cpu:
    push rbx
    mov r8, rcx
    xor eax, eax
    cpuid
    mov dword ptr [r8], ebx
    mov dword ptr [r8 + 4], edx
    mov dword ptr [r8 + 8], ecx
    mov eax, 1
    cpuid
    mov dword ptr [r8 + 12], eax
    mov dword ptr [r8 + 16], ecx
    mov dword ptr [r8 + 24], edx
    mov eax, 7
    xor ecx, ecx
    cpuid
    mov dword ptr [r8 + 20], ebx
    mov eax, 0x80000001
    cpuid
    mov dword ptr [r8 + 28], ecx
    pop rbx
    ret

.globl require_avx
require_avx:
    vzeroupper
    ret

.globl require_avx2
require_avx2:
    vpaddd ymm0, ymm0, ymm0
    vzeroupper
    ret

.globl require_fma
require_fma:
    vxorps xmm0, xmm0, xmm0
    vfmadd132ps ymm0, ymm0, ymm0
    vzeroupper
    ret

.globl require_bmi2
require_bmi2:
    xor eax, eax
    pext eax, eax, eax
    ret
