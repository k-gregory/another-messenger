#include "dialerwindow.h"

#include <QtDebug>
#include <QWidget>
#include <QSizePolicy>
#include <QString>

#include <QNetworkDatagram>

DialerWindow::DialerWindow(quint16 port, QWidget* parent)
 : QMainWindow (parent)
{
    initUi();
    socket = new QUdpSocket(this);
    socket->bind(QHostAddress("0.0.0.0"), port);
    connect(socket, &QUdpSocket::readyRead, this, &DialerWindow::dataArrived);
    connect(dialBtn, &QPushButton::clicked, this, &DialerWindow::dialInitiated);
}


void DialerWindow::initUi(){
    layout = new QGridLayout;

    ipLabel = new QLabel(tr("IP address"));
    ipEdit = new QLineEdit;
    ipEdit->setText("0.0.0.0");
    layout->addWidget(ipLabel, 0, 0);
    layout->addWidget(ipEdit, 1, 0);

    portLabel = new QLabel(tr("Port"));
    portEdit = new QLineEdit;
    portEdit->setText("4567");
    layout->addWidget(portLabel, 0, 1);
    layout->addWidget(portEdit, 1, 1);


    chatLog = new QListWidget;
    msgEdit = new QLineEdit;
    dialBtn = new QPushButton(tr("Send"));
    layout->addWidget(chatLog, 2, 0, 1, 2);
    layout->addWidget(msgEdit, 3, 0);
    layout->addWidget(dialBtn, 3, 1);

    QWidget *wgt = new QWidget;
    wgt->setLayout(layout);
    setCentralWidget(wgt);
}

void DialerWindow::dialInitiated(){
    auto ip = QHostAddress(ipEdit->text());
    auto port = static_cast<quint16>(portEdit->text().toInt());
    QByteArray ba;
    auto text = msgEdit->text();
    ba.append(text);
    chatLog->addItem(QString("You: ") + text);
    msgEdit->clear();
    socket->writeDatagram(ba, ip, port);
}

void DialerWindow::dataArrived(){
    auto dg = socket->receiveDatagram().data();
    chatLog->addItem(QString("Other: ").append(dg));
}
