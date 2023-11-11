// Tomasulo's Algorithm in C

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <assert.h>

typedef enum Opcode {
    FADD, FSUB, FMUL, FDIV, FLD, FST,
    ADD, SUB, MUL, DIV, LD, ST,
    BEQ, BNE
} Opcode;

typedef enum Stage {
    ISSUE, EXECUTE, MEM_ACCESS, WRITE_RESULT, COMMIT
} Stage;

void opcode_print(Opcode opcode) {
    switch (opcode) {
    case FADD:
        printf("fadd");
        break;
    case FSUB:
        printf("fsub");
        break;
    case FMUL:
        printf("fmul");
        break;
    case FDIV:
        printf("fdiv");
        break;
    case FLD:
        printf("fld");
        break;
    case FST:
        printf("fst");
        break;
    case ADD:
        printf("add");
        break;
    case SUB:
        printf("sub");
        break;
    case MUL:
        printf("mul");
        break;
    case DIV:
        printf("div");
        break;
    case LD:
        printf("ld");
        break;
    case ST:
        printf("st");
        break;
    case BEQ:
        printf("beq");
        break;
    case BNE:
        printf("bne");
        break;
    }
}

typedef struct Operand {
    union {
        uint8_t reg_num;
        uint32_t imm;
        struct {
            // The register number of the indirect register
            uint8_t indirect_reg_num;
            // The offset from the indirect register
            uint32_t offset;
            // The predicted address of the indirect operation (given to the simulator)
            uint64_t addr;
        };

        // The reservation station number that will produce the value
        // for this operand
        uint64_t reservation_station_entry;
    };

    enum {
        // A floating point register
        FREG,
        // An integer register
        IREG,
        // An immediate value
        IMM,
        // An indirect memory address
        INDIRECT,
        // A reservation station entry
        RESERVATION_STATION_ENTRY,
        // An unused operand
        UNUSED
    } type;
} Operand;

void operand_print(Operand operand) {
    switch (operand.type) {
    case FREG:
        printf("f%d", operand.reg_num);
        break;
    case IREG:
        printf("x%d", operand.reg_num);
        break;
    case IMM:
        printf("%d", operand.imm);
        break;
    case INDIRECT:
        printf("%d(x%d):%ld", operand.offset, operand.indirect_reg_num, operand.addr);
        break;
    case RESERVATION_STATION_ENTRY:
        printf("#%ld", operand.reservation_station_entry);
        break;

    case UNUSED:
    default:
        break;
    }
}

bool operand_is_used(Operand operand) {
    return operand.type != UNUSED;
}

#define F(n) ((Operand){ .type = FREG, .reg_num = n })
#define X(n) ((Operand){ .type = IREG, .reg_num = n })
#define I(n) ((Operand){ .type = IMM,  .imm = n })
#define M(r, o, a) ((Operand){ .type = INDIRECT, .indirect_reg_num = r.reg_num, .offset = o, .addr = a })

const Operand UNUSED_OPERAND = { .type = UNUSED };

typedef struct Op {
    Opcode opcode;
    // The operands of the operation
    Operand dst, src[2];

    // The stage of the operation
    Stage stage;
} Op;

void op_print(Op op) {
    opcode_print(op.opcode);
    printf(" ");
    operand_print(op.dst);
    printf(", ");
    if (operand_is_used(op.src[0])) {
        operand_print(op.src[0]);
        printf(", ");
    }
    if (operand_is_used(op.src[1])) {
        operand_print(op.src[1]);
    }
}

Op op_new_fld(Operand dst, Operand src) {
    Op op;
    op.opcode = FLD;
    op.dst = dst;
    assert(dst.type == FREG);
    op.src[0] = src;
    assert(src.type == INDIRECT);
    op.src[1] = UNUSED_OPERAND;
    return op;
}

bool op_only_uses_reservation_station_entries(Op op) {
    for (int i = 0; i < 3; i++) {
        if (op.dst.type != RESERVATION_STATION_ENTRY) {
            return false;
        }
        if (op.src[i].type != RESERVATION_STATION_ENTRY) {
            return false;
        }
    }
    return true;
}


typedef struct ReorderBuffer {
    // The number of entries in the reorder buffer
    uint64_t size;
    // The number of entries in the reorder buffer that are currently in use
    uint64_t used;
    // The reorder buffer entries
    uint64_t *entries;
} ReorderBuffer;

ReorderBuffer reorder_buffer_new(uint64_t size) {
    ReorderBuffer reorder_buffer;
    reorder_buffer.size = size;
    reorder_buffer.used = 0;
    reorder_buffer.entries = calloc(size, sizeof(uint64_t));
    return reorder_buffer;
}

typedef struct CommonDataBus {
    // The reorder buffer entry that is currently on the common data bus
    uint64_t reorder_buffer_entry;
    // Is the common data bus currently in use?
    bool in_use;
} CommonDataBus;


typedef struct ReservationStation {
    // The number of entries in the reservation station
    uint64_t size;
    // The number of entries in the reservation station that are currently in use
    uint64_t used;
    // The reservation station entries
    Op *entries;
} ReservationStation;

int main(int argc, char *argv[]) {
    Op op = op_new_fld(F(3), M(X(2), 5, 0x1000));
    op_print(op);
    printf("\n");
}