/*
Copyright (C) 2020  Paul Kremer

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "thumbcreator.h"
#include "stl2thumbnail.h"

#include <QImage>
#include <QString>
#include <QMimeDatabase>

Q_LOGGING_CATEGORY(LOG_STL_THUMBS, "STLModelThumbs")

extern "C"
{
    // Factory method
    Q_DECL_EXPORT ThumbCreator *new_creator()
    {
        return new StlThumbCreator();
    }
};

StlThumbCreator::StlThumbCreator()
{
}

struct PicContainer
{
    s2t::PictureBuffer buffer;
};

void cleanup(void *data)
{
    auto container = static_cast<PicContainer *>(data);
    s2t::free_picture_buffer(&container->buffer);
    delete container;
}

bool StlThumbCreator::create(const QString &path, int width, int height, QImage &img)
{
    auto mime_type = QMimeDatabase().mimeTypeForFile(path);

    s2t::PictureBuffer pic;

    if (mime_type.inherits("model/stl"))
    {
        s2t::RenderSettings settings;
        settings.width = width;
        settings.height = height;
        settings.timeout = 20000; // 20s
        settings.size_hint = height >= 256;

        // render
        pic = s2t::render_stl(path.toStdString().c_str(), settings);
    }
    else if (mime_type.inherits("text/x.gcode"))
    {
        pic = s2t::extract_gcode_preview(path.toStdString().c_str(), width, height);
    }
    else if (mime_type.inherits("model/3mf"))
    {
        pic = s2t::extract_3mf_preview(path.toStdString().c_str(), width, height);
    }

    // failed?
    if (!pic.data)
        return false;

    // save the buffer in a container that stays around till it gets cleaned up
    // once 'img' goes out of scope
    auto container = new PicContainer{pic};

    // QImage owns the buffer and it has to stay valid throughout the life of the QImage
    img = QImage(pic.data, pic.width, pic.height, pic.stride, QImage::Format_RGBA8888, cleanup, container);

    return true;
}
