#pragma once

#include "arrow/c/abi.h"
#include "arrow/c/bridge.h"
#include "arrow/c/helpers.h"
#include "arrow/util/logging.h"
#include "arrow/record_batch.h"


// just for test
// copied from arrow/c/bridge.cc
class ArrayStreamBatchReader : public arrow::RecordBatchReader {
public:
    explicit ArrayStreamBatchReader(std::shared_ptr<arrow::Schema> schema,
                                    struct ArrowArrayStream *stream)
            : schema_(std::move(schema)) {
        ArrowArrayStreamMove(stream, &stream_);
                DCHECK(!ArrowArrayStreamIsReleased(&stream_));
    }

    ~ArrayStreamBatchReader() override {
        if (!ArrowArrayStreamIsReleased(&stream_)) {
            ArrowArrayStreamRelease(&stream_);
        }
                DCHECK(ArrowArrayStreamIsReleased(&stream_));
    }

    std::shared_ptr<arrow::Schema> schema() const override { return schema_; }

    arrow::Status ReadNext(std::shared_ptr<arrow::RecordBatch> *batch) override {
        struct ArrowArray c_array;
        if (ArrowArrayStreamIsReleased(&stream_)) {
            return arrow::Status::Invalid(
                    "Attempt to read from a reader that has already been closed");
        }
        RETURN_NOT_OK(StatusFromCError(stream_.get_next(&stream_, &c_array)));
        if (ArrowArrayIsReleased(&c_array)) {
            // End of stream
            batch->reset();
            return arrow::Status::OK();
        } else {
            return ImportRecordBatch(&c_array, schema_).Value(batch);
        }
    }

    arrow::Status Close() override {
        if (!ArrowArrayStreamIsReleased(&stream_)) {
            ArrowArrayStreamRelease(&stream_);
        }
        return arrow::Status::OK();
    }

    static arrow::Result<std::shared_ptr<RecordBatchReader>> Make(
            struct ArrowArrayStream *stream) {
        if (ArrowArrayStreamIsReleased(stream)) {
            return arrow::Status::Invalid("Cannot import released ArrowArrayStream");
        }
        std::shared_ptr<arrow::Schema> schema;
        struct ArrowSchema c_schema = {};
        auto status = StatusFromCError(stream, stream->get_schema(stream, &c_schema));
        if (status.ok()) {
            status = arrow::ImportSchema(&c_schema).Value(&schema);
        }
        if (!status.ok()) {
            ArrowArrayStreamRelease(stream);
            return status;
        }
        return std::make_shared<ArrayStreamBatchReader>(std::move(schema), stream);
    }

private:
    arrow::Status StatusFromCError(int errno_like) const {
        return StatusFromCError(&stream_, errno_like);
    }

    static arrow::Status StatusFromCError(struct ArrowArrayStream *stream, int errno_like) {
        if (ARROW_PREDICT_TRUE(errno_like == 0)) {
            return arrow::Status::OK();
        }
        arrow::StatusCode code;
        switch (errno_like) {
            case EDOM:
            case EINVAL:
            case ERANGE:
                code = arrow::StatusCode::Invalid;
                break;
            case ENOMEM:
                code = arrow::StatusCode::OutOfMemory;
                break;
            case ENOSYS:
                code = arrow::StatusCode::NotImplemented;
                break;
            default:
                code = arrow::StatusCode::IOError;
                break;
        }
        const char *last_error = stream->get_last_error(stream);
        return {code, last_error ? std::string(last_error) : ""};
    }

    mutable struct ArrowArrayStream stream_;
    std::shared_ptr<arrow::Schema> schema_;
};