#include <catch2/catch_test_macros.hpp>

#include "car_bridge/lib.h"
#include "arrow_array_stream_reader.h"
#include "arrow/record_batch.h"
#include "arrow/ipc/json_simple.h"
#include "arrow/c/abi.h"
#include "arrow/c/bridge.h"
#include <memory>

extern "C" {
ArrowArrayStream *process(void *handler, ArrowArrayStream *input);
}

using std::make_unique;

TEST_CASE("create handler") {
    auto handler = make_unique<rust::Box<car::DynHandler>>(car::create_handler("NoOp"));
    REQUIRE(handler);
    REQUIRE_THROWS(car::create_handler("unknown"));
}

TEST_CASE("process") {
    auto handler = make_unique<rust::Box<car::DynHandler>>(car::create_handler("NoOp"));
    const int length = 3;

    auto field1 = field("f1", arrow::utf8());
    auto field2 = field("f2", arrow::int64());
    auto field3 = field("f3", arrow::float64());

    auto schema = arrow::schema({field1, field2, field3});

    auto array1 =
            arrow::ipc::internal::json::ArrayFromJSON(arrow::utf8(), R"(["v1", "v2", "v3"])")
                    .ValueOrDie();
    auto array2 =
            arrow::ipc::internal::json::ArrayFromJSON(arrow::int64(), R"([1, 2, 3])").ValueOrDie();
    auto array3 = arrow::ipc::internal::json::ArrayFromJSON(arrow::float64(), R"([1.1, 2.2, 3.3])")
            .ValueOrDie();

    auto batch1 = arrow::RecordBatch::Make(schema, length, {array1, array2, array3});
    std::vector<std::shared_ptr<arrow::RecordBatch>> batches{batch1, batch1, batch1};
    auto input_reader = arrow::RecordBatchReader::Make(batches, schema).ValueOrDie();
    struct ArrowArrayStream c_stream;
    REQUIRE(arrow::ExportRecordBatchReader(input_reader, &c_stream).ok());
    auto out = process(handler.get(), &c_stream);
    REQUIRE(out);
    auto output_reader = ArrayStreamBatchReader::Make(out).ValueOrDie();
    std::shared_ptr<arrow::RecordBatch> batch;
    REQUIRE(output_reader->ReadNext(&batch).ok());
    REQUIRE(batch->num_rows() == 3);
    REQUIRE(output_reader->ReadNext(&batch).ok());
    REQUIRE(batch);
}